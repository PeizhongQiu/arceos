mod apic;
mod entry;
mod dtables;
mod uart16550;
pub mod mem;
pub mod misc;
pub mod time;
pub mod percpu;
use axlog::ax_println as println;

#[cfg(feature = "irq")]
pub mod irq {
    pub use super::apic::*;
}

pub mod console {
    pub use super::uart16550::*;
}
use core::sync::atomic::{AtomicI32, AtomicU32, Ordering};
static INIT_EARLY_OK: AtomicU32 = AtomicU32::new(0);


extern "C" {
    fn rust_main_type1_5(cpu_id: u32, linux_sp: usize) -> i32;
    fn rust_main(cpu_id: usize, dtb: usize) -> !;
    #[cfg(feature = "smp")]
    fn rust_main_secondary_type1_5(cpu_id: usize) -> u32;
}



fn primary_init_early(cpu_id: u32) {
    crate::mem::clear_bss();
    crate::cpu::init_primary(cpu_id as usize);
    self::uart16550::init();
    self::dtables::init_primary();
    self::time::init_early();
    println!("primary_init_early OK!!!");
    INIT_EARLY_OK.store(1, Ordering::Release);
}

// hypervisor start
extern "sysv64" fn rust_entry_hv(cpu_id: u32, linux_sp: usize) -> i32 {
    if cpu_id == 0 {
        primary_init_early(cpu_id);
    } else {
        while INIT_EARLY_OK.load(Ordering::Acquire) < 1 {
            core::hint::spin_loop();
        }
    }
    let ret = unsafe { rust_main_type1_5(cpu_id, linux_sp) };
    // println!(
    //     "CPU {} return back to driver with code {}.",
    //     cpuid, ret
    // );
    ret
}


/// Initializes the platform devices for the primary CPU.
pub fn platform_init() {
    self::apic::init_primary();
    self::time::init_primary();
}

