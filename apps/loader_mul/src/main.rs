#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(asm_const)]

extern crate alloc;


use alloc::vec::Vec;

#[cfg(feature = "axstd")]
use axstd::{println,print};
use xmas_elf::symbol_table::Entry;
const PLASH_START: usize = 0x22000000;
const RUN_START: usize = 0x4010_0000;

pub struct APP_PT_SV39 ([u64; 512]);

impl APP_PT_SV39 {
    /// Create a new page table
    pub const fn new(id: u64) -> Self {
        let mut newPageTable = APP_PT_SV39 ([0; 512]);
        // 0x8000_0000..0xc000_0000, VRWX_GAD, 1G block
        newPageTable.0[2] = (0x80000 << 10) | 0xef;
        // 0xffff_ffc0_8000_0000..0xffff_ffc0_c000_0000, VRWX_GAD, 1G block
        newPageTable.0[0x102] = (0x80000 << 10) | 0xef;
    
        // 0x0000_0000..0x4000_0000, VRWX_GAD, 1G block
        newPageTable.0[0] = (0x00000 << 10) | 0xef;
    
        // For App aspace!
        // 0x4000_0000..0x8000_0000, VRWX_GAD, 1G block
        newPageTable.0[1] = ((0x1 + id*2) << 29) | 0xef;
        newPageTable
    }
}

unsafe fn switch_app_aspace(to:APP_PT_SV39) {
    use riscv::register::satp;
    let page_table_root = to.0.as_ptr() as usize - axconfig::PHYS_VIRT_OFFSET;
    satp::set(satp::Mode::Sv39, 0, page_table_root >> 12);
    riscv::asm::sfence_vma_all();
}

#[link_section = ".data.app_page_table"]
static mut APP_PAGE_TABLE: [APP_PT_SV39; 2] = [APP_PT_SV39::new(0), APP_PT_SV39::new(1)];

static mut MAIN_ENTRY: [usize; 16] = [0; 16];

pub extern "C" fn puts(str: &[u8]) {
    let mut i:usize = 0;
    loop {
        if str[i] != 0 {
            print!("{}",str[i] as char);
        } else {
            print!("\n");
            break;
        }
        i = i + 1;
    }
}

pub extern "C" fn __libc_start_main() {
    println!("This is __libc_start_main function!");
    // run main
    println!(" Call main()!");
    unsafe { core::arch::asm!("
        mv      t2, {run_start}
        jalr    t2",
        run_start = in(reg) MAIN_ENTRY[0],
    )}   
    // exit
    // axhal::misc::terminate();
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let num_app_ptr = PLASH_START as usize as *const usize;
    let num_app = unsafe { num_app_ptr.read_volatile() };
    let app_start: Vec<usize>  =  unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app * 2).to_vec()};
        
    println!("Load payload ...");
    for i in 0..num_app {
        let start = (app_start[i*2]+PLASH_START) as *const u8;
        let size = app_start[i*2+1];
        println!("{} {}", start as usize, size);
        let elf_data = unsafe { core::slice::from_raw_parts(start, size)};
        

        let run_code = unsafe {
            core::slice::from_raw_parts_mut(RUN_START as *mut u8, size)
        };
        run_code.copy_from_slice(elf_data);
        // println!("run code {:?}; address [{:?}]", run_code, run_code.as_ptr());

        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");

        let entry = elf.header.pt2.entry_point() as usize;
        println!("entry : {:x}", entry);
        let entry = entry + RUN_START;
        println!("entry : {:x}", entry);

        let rela = elf.find_section_by_name(".rela.plt").unwrap();
        let rela_data = rela.get_data(&elf).unwrap();
        
        
        if let xmas_elf::sections::SectionData::Rela64(data) = rela_data {
            for iter in data.iter() {
                let index = iter.get_symbol_table_index();
                let dynsym = elf.find_section_by_name(".dynsym").unwrap();
                let dynsym_data = dynsym.get_data(&elf).unwrap();
                if let xmas_elf::sections::SectionData::DynSymbolTable64(sym_data) = dynsym_data {
                    
                    let name = sym_data[index as usize].get_name(&elf).unwrap();
                    println!("{} {} {}",index, name, iter.get_offset());
                    match name {
                        "puts" => {
                            let offset_puts = puts as *const () as usize;
                            println!("offset_puts : {:x}", offset_puts);
                            let address_puts = unsafe {
                                core::slice::from_raw_parts_mut((RUN_START + iter.get_offset() as usize) as *mut usize, 1)
                            };
                            address_puts[0] = offset_puts;
                        }
                        "__libc_start_main" => {
                            let offset_libc_start_main = __libc_start_main as *const () as usize;
                            println!("offset_libc_start_main : {:x}", offset_libc_start_main);
                            let address_libc_start_main = unsafe {
                                core::slice::from_raw_parts_mut((RUN_START + iter.get_offset() as usize) as *mut usize, 1)
                            };
                            address_libc_start_main[0] = offset_libc_start_main;
                        }
                        _ => {}
                    }
                } else {
                    println!("no dynsym_data");
                }
                
            }
            
        } else {
            println!("no rela_date");
        }

        let sym = elf.find_section_by_name(".symtab").unwrap();
        let sym_data = sym.get_data(&elf).unwrap();
        if let xmas_elf::sections::SectionData::SymbolTable64(data) = sym_data {
            for iter in data.iter() {
                if iter.get_name(&elf).unwrap() == "main" {
                    let main_entry = iter.value();
                    println!("{:x}",main_entry);
                    unsafe { MAIN_ENTRY[i] = RUN_START + main_entry as usize; }
                }
            }
        }
        

        println!("Execute app ...");
        // execute app

        unsafe { core::arch::asm!("
            mv      t2, {run_start}
            jalr    t2",
            run_start = in(reg) entry,
        )}   
    }
    println!("Load payload ok!");
}
