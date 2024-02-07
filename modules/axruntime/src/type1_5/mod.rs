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
use core::sync::atomic::{AtomicI32, AtomicU32, Ordering};
use axhal::mem;

mod config;
mod header;
mod error;
mod percpu;
use config::HvSystemConfig;
use header::HvHeader;
mod consts;
// mod cell;

use error::{HvError, HvErrorNum, HvResult};

// use axhal::rust_entry;

// use percpu::PerCpu;

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

// fn primary_init_early_linux() -> HvResult {
//     let system_config = HvSystemConfig::get();
//     println!(
//         "\nInitializing hypervisor...\n"
//     );

//     // memory::init_heap();    // 分配一段 32 MB 内存，初始化 PHYS_VIRT_OFFSET
//     system_config.check()?; // 检查 signature 和 revision
//     println!("Hypervisor header: {:#x?}", HvHeader::get());
//     println!("System config: {:#x?}", system_config);

//     // memory::init_frame_allocator(); // 初始化物理内存分配器
//     // memory::init_hv_page_table()?;  // 初始化 HV_PT，hypervisor 用的页表
//     cell::init()?;  // 初始化 ROOT_CELL，包括页表和 CellConfig

//     INIT_EARLY_OK.store(1, Ordering::Release);
//     Ok(())
// }

fn primary_init_late() {
    info!("Primary CPU init late...");
    // Do nothing...
    INIT_LATE_OK.store(1, Ordering::Release);
}


pub fn start_type1_5(cpu_id: u32, linux_sp: usize) -> HvResult {
    let is_primary = cpu_id == 0;  
    let online_cpus = HvHeader::get().online_cpus; // 2
    
    // one core for arceos, one core for linux
    if is_primary {
        // primary_init_early_linux()?;
    } else { 
        // main();
    }
    // wait_for_counter(&INIT_EARLY_OK, online_cpus)?;
    // cpu_data.init(linux_sp, cell::root_cell())?;    
    // println!(
    //     "[main]: cpudata: {:#?}",
    //     cpu_data
    // );
    // println!("CPU {} init OK.", cpu_id);
    // INITED_CPUS.fetch_add(1, Ordering::SeqCst);
    // wait_for_counter(&INITED_CPUS, online_cpus)?;

    // if is_primary { 
    //     primary_init_late();
    // } else {    // 等待 primary_init_late 完成
    //     // wait_for_counter(&INIT_LATE_OK, 1)?
    // }

    // cpu_data.activate_vmm()
    Ok(())
}

