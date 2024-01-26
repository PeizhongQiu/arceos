mod apic;
mod entry;
mod dtables;
mod uart16550;
pub mod mem;
pub mod misc;
pub mod time;
use axlog::ax_println as println;

#[cfg(feature = "irq")]
pub mod irq {
    pub use super::apic::*;
}

pub mod console {
    pub use super::uart16550::*;
}

extern "C" {
    fn rust_main_type1_5(cpu_id: u32, linux_sp: usize) -> u32;
    fn rust_main(cpu_id: usize, dtb: usize) -> !;
    #[cfg(feature = "smp")]
    fn rust_main_secondary_type1_5(cpu_id: usize) -> u32;
}

fn current_cpu_id() -> usize {
    match raw_cpuid::CpuId::new().get_feature_info() {
        Some(finfo) => finfo.initial_local_apic_id() as usize,
        None => 0,
    }
}

// hypervisor start
extern "sysv64" fn rust_entry_hv(cpuid: u32, linux_sp: usize) -> u32 {
    let ret = unsafe { rust_main_type1_5(cpuid, linux_sp) };
    println!(
        "CPU {} return back to driver with code {}.",
        cpuid, ret
    );
    ret
}


/// app start
pub fn rust_entry() {
    crate::mem::clear_bss();
    crate::cpu::init_primary(current_cpu_id());
    self::uart16550::init();
    self::dtables::init_primary();
    self::time::init_early();
    unsafe { rust_main(current_cpu_id(), 0) };
}
/// Initializes the platform devices for the primary CPU.
pub fn platform_init() {
    self::apic::init_primary();
    self::time::init_primary();
}

