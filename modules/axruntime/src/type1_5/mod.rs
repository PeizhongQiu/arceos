#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(dead_code))]
#![feature(asm_sym)]
#![feature(asm_const)]
#![feature(lang_items)]
#![feature(concat_idents)]
#![feature(naked_functions)]
#![allow(unaligned_references)]

use alloc::string::String;
use axlog::ax_println as println;
use memory_addr::PhysAddr;
use core::sync::atomic::{AtomicI32, AtomicU32, Ordering};
use axhal::mem;
use axhal::{
    mem::VirtAddr,
    paging::{MappingFlags, PageTable},
};
use axhal::mem::virt_to_phys;
use axhal::mem::phys_to_virt;
use spin::{Once, RwLock};
use config::MemFlags;

mod config;
mod header;
mod error;
mod percpu;
use config::HvSystemConfig;
use header::HvHeader;
mod consts;
mod cell;
mod mm;

use error::{HvError, HvErrorNum, HvResult};

use crate::type1_5::cell::root_cell;

/// Page table used for hypervisor.
static HV_PT: Once<RwLock<PageTable>> = Once::new();

static INITED_CPUS: AtomicU32 = AtomicU32::new(0);
static INIT_EARLY_OK: AtomicU32 = AtomicU32::new(0);
static INIT_LATE_OK: AtomicU32 = AtomicU32::new(0);
static ERROR_NUM: AtomicI32 = AtomicI32::new(0);

fn has_err() -> bool {
    ERROR_NUM.load(Ordering::Acquire) != 0
}

fn wait_for(condition: impl Fn() -> bool) -> HvResult {
    while !has_err() && condition() {
        core::hint::spin_loop();
    }
    if has_err() {
        Err(HvError::new(HvErrorNum::EBUSY, file!(), line!(), column!(), Some(String::from("Other cpu init failed!"))))
    } else {
        Ok(())
    }
}

fn wait_for_counter(counter: &AtomicU32, max_value: u32) -> HvResult {
    wait_for(|| counter.load(Ordering::Acquire) < max_value)
}

pub fn init_phys_virt_offset() {
    // Set PHYS_VIRT_OFFSET early.
    unsafe {
        axhal::PHYS_VIRT_OFFSET =
            consts::HV_BASE - HvSystemConfig::get().hypervisor_memory.phys_start as usize
    };
    unsafe {info!("init_PHYS_VIRT_OFFSET: {:x}",axhal::PHYS_VIRT_OFFSET)};
}

pub fn init_cell() -> usize{
    cell::init();
    info!("{:#x?}", root_cell().gpm);
    root_cell().gpm.nest_page_table_root()
}

pub fn init_allocator_type1_5() {
    let mem_pool_start = consts::free_memory_start();
    let mem_pool_end = consts::hv_end().align_down_4k();

    let mem_pool_size = mem_pool_end.as_usize() - mem_pool_start.as_usize();
    info!("global_init start:{:x}, end:{:x}.",mem_pool_start,mem_pool_end);
    axalloc::global_init(mem_pool_start.as_usize(), mem_pool_size);
    
    let header = HvHeader::get();
    let sys_config = HvSystemConfig::get();
    let cell_config = sys_config.root_cell.config();
    let hv_phys_start = sys_config.hypervisor_memory.phys_start as usize;
    let hv_phys_size = sys_config.hypervisor_memory.size as usize;
    info!("create PageTable.");
    let mut page_table = PageTable::try_new().unwrap();
    
    page_table.map_region(
        consts::HV_BASE.into(),
        hv_phys_start.into(),
        header.core_size,
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
        false,
    );
    info!("map_region {:x},{:x},{:x},{:?},{}.",
    consts::HV_BASE,
    hv_phys_start,
    header.core_size,
    MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
    false);
    page_table.map_region(
        (consts::HV_BASE + header.core_size).into(),
        (hv_phys_start + header.core_size).into(),
        hv_phys_size - header.core_size,
        MappingFlags::READ | MappingFlags::WRITE,
        false,
    );
    info!("map_region {:x},{:x},{:x},{:?},{}.",
    consts::HV_BASE + header.core_size,
        hv_phys_start + header.core_size,
        hv_phys_size - header.core_size,
        MappingFlags::READ | MappingFlags::WRITE,
        false);
    // Map all guest RAM to directly access in hypervisor.
    for region in cell_config.mem_regions() {
        let flags = region.flags; 
        if flags.contains(MemFlags::DMA) {
            let hv_virt_start = phys_to_virt(PhysAddr::from(region.virt_start as _));
            if hv_virt_start < VirtAddr::from(region.virt_start as _) {
                let virt_start = region.virt_start;
                panic!(
                        "Guest physical address {:#x} is too large",
                        virt_start
                );
            }
            page_table.map_region(
                hv_virt_start,
                PhysAddr::from(region.phys_start as _),
                region.size as usize,
                MappingFlags::READ | MappingFlags::WRITE,
                false
            );
            info!("map_region {:x},{:x},{:x},{:?},{}.",
            hv_virt_start.as_usize(),
            region.phys_start as usize,
            region.size as usize,
            MappingFlags::READ | MappingFlags::WRITE,
            false);
        }
    }
    info!("Hypervisor page table init end.");
    // info!("Hypervisor virtual memory set: {:#x?}", page_table);
    
    HV_PT.call_once(|| RwLock::new(page_table));
}

pub fn activate_hv_pt() {
    info!("activate_hv_pt!!!");
    let page_table = HV_PT.get().expect("Uninitialized hypervisor page table!");
    unsafe { axhal::arch::write_page_table_root(page_table.read().root_paddr()) };
}