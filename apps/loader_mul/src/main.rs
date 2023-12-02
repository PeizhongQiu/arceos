#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(asm_const)]

extern crate alloc;

use alloc::vec::Vec;
use axalloc::global_allocator;
use axtask::spawn_ptr;
use axtask::AxTaskRef;
use core::ffi::{c_char, c_int, CStr};

use axhal::{
    mem::VirtAddr,
    paging::{MappingFlags, PageTable},
};
#[cfg(feature = "axstd")]
use axstd::{print, println};
use xmas_elf::symbol_table::Entry;
const PLASH_START: usize = 0x22000000 + axconfig::PHYS_VIRT_OFFSET;

pub extern "C" fn puts(str: &CStr) -> usize {
    match str.to_str() {
        Ok(_) => {
            println!("{}", str.to_str().unwrap());
            str.to_str().unwrap().len()
        }
        _ => 0,
    }
}

pub extern "C" fn __libc_start_main(main: fn(argc: c_int, argv: &&c_char) -> c_int) {
    println!("This is __libc_start_main function!");
    axtask::exit(main(0, &&0));
}

fn init_app_page_table() -> PageTable {
    let mut page_table = PageTable::try_new().unwrap();
    // 0xffff_ffc0_8000_0000..0xffff_ffc0_c000_0000, VRWX_GAD, 1G block
    let _ = page_table.map_region(
        0xffff_ffc0_8000_0000.into(),
        0x8000_0000.into(),
        0x4000_0000,
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
        true,
    );
    page_table
}

fn va_to_kva(page_table: &PageTable, va: u64) -> usize {
    let (pa, _, _) = page_table
        .query((va as usize).into())
        .unwrap();
    let pa: usize = pa.into();
    pa + axconfig::PHYS_VIRT_OFFSET
}

fn load_elf(num_app_ptr: *const usize, i: usize) -> AxTaskRef {
    let mut page_table = init_app_page_table();
    let app_start: Vec<usize> =
        unsafe { core::slice::from_raw_parts(num_app_ptr.add(1 + i * 2), 2).to_vec() };

    let start = app_start[0] + PLASH_START;
    let size = app_start[1];
    println!("{:x} {:x}", start, size);

    let elf_data = unsafe { core::slice::from_raw_parts(start as *const u8, size) };

    let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
    let elf_header = elf.header;
    let magic = elf_header.pt1.magic;
    assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");

    let ph_count = elf_header.pt2.ph_count();
    // let mut run_start: usize = 0;

    for i in 0..ph_count {
        let ph = elf.program_header(i).unwrap();
        if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
            let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
            let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
            let start_aligned_va = start_va.align_down_4k();
            let end_aligned_va = end_va.align_up_4k();
            let offset = start_va.align_offset_4k();
            let num_pages = (usize::from(end_aligned_va) - usize::from(start_aligned_va)) / 4096;
            println!(
                "{:x?} {:x?} {:x?} {:x?} {:x?} {}",
                start_va, end_va, start_aligned_va, end_aligned_va, offset, num_pages
            );
            let pages_kva = global_allocator().alloc_pages(num_pages, 4096).unwrap();
            let run_start = usize::from(pages_kva) + offset;
            println!("{:x} {:x}", pages_kva, run_start);

            let mut map_perm = MappingFlags::empty();
            let ph_flags = ph.flags();
            if ph_flags.is_read() {
                map_perm |= MappingFlags::READ;
            }
            if ph_flags.is_write() {
                map_perm |= MappingFlags::WRITE;
            }
            if ph_flags.is_execute() {
                map_perm |= MappingFlags::EXECUTE;
            }
            let va = start_aligned_va;
            let pa = pages_kva - axconfig::PHYS_VIRT_OFFSET;
            page_table.map_region(va.into(), pa.into(), 4096 * num_pages, map_perm, false);
            
            let load_bin = unsafe {
                core::slice::from_raw_parts(
                    (start + ph.offset() as usize) as *const u8,
                    ph.file_size() as usize,
                )
            };
            let load_code = unsafe {
                core::slice::from_raw_parts_mut(run_start as *mut u8, ph.file_size() as usize)
            };
            load_code.copy_from_slice(load_bin);
        }
    }

    let rela = elf.find_section_by_name(".rela.plt").unwrap();
    let rela_data = rela.get_data(&elf).unwrap();

    if let xmas_elf::sections::SectionData::Rela64(data) = rela_data {
        for iter in data.iter() {
            let index = iter.get_symbol_table_index();
            let dynsym = elf.find_section_by_name(".dynsym").unwrap();
            let dynsym_data = dynsym.get_data(&elf).unwrap();
            if let xmas_elf::sections::SectionData::DynSymbolTable64(sym_data) = dynsym_data {
                let name = sym_data[index as usize].get_name(&elf).unwrap();
                println!("{} {} {:x}", index, name, iter.get_offset());
                let mut offset: usize = 0;
                match name {
                    "puts" => {
                        offset = puts as usize;
                        println!("offset_puts : {:x}", offset);
                    }
                    "__libc_start_main" => {
                        offset = __libc_start_main as usize;
                        println!("offset_libc_start_main : {:x}", offset);
                    }
                    _ => {}
                }
                
                let kva = va_to_kva(&page_table, iter.get_offset());
                let link_dest = unsafe { &mut *(kva as *mut usize) };
                *link_dest = offset;
            } else {
                println!("no dynsym_data");
            }
        }
    } else {
        println!("no rela_date");
    }

    let rela_dyn = elf.find_section_by_name(".rela.dyn").unwrap();
    let rela_dyn_data = rela_dyn.get_data(&elf).unwrap();
    // println!("{:?}",rela_dyn.get_type().unwrap());
    if let xmas_elf::sections::SectionData::Rela64(data) = rela_dyn_data {
        for iter in data.iter() {
            //let link_addr = iter.
            let link_vaddr = iter.get_addend() as usize;
            let kva = va_to_kva(&page_table, iter.get_offset());
            let link_dest = unsafe { &mut *(kva as *mut usize) };
            *link_dest = link_vaddr;
        }
    } else {
        println!("no rela_dyn_data");
    }

    let entry = elf.header.pt2.entry_point() as usize;
    println!("entry : {:x}", entry);
    let page_table_pa: usize = page_table.root_paddr().into();
    println!("set page table, pa: {:x}", page_table_pa);
    let satp = (8 << 60) | (0 << 44) | (page_table_pa >> 12);
    println!("satp: {:x}", satp);
    let inner = spawn_ptr(entry, "hello".into(), 4096, satp, page_table);
    inner
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let num_app_ptr = PLASH_START as usize as *const usize;
    let num_app = unsafe { num_app_ptr.read_volatile() };

    let mut tasks = Vec::new();

    for i in 0..num_app {
        println!("Load payload ...");
        let inner = load_elf(num_app_ptr, i);

        println!("Execute app ...");
        tasks.push(inner);
    }
    tasks.into_iter().for_each(|t| {
        t.join();
    });
    println!("Load payload ok!");
}
