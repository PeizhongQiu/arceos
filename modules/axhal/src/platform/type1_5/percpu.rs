
use core::sync::atomic::{AtomicU32, Ordering};
static ENTERED_CPUS: AtomicU32 = AtomicU32::new(0);

// pub fn current_cpu_id() -> u32{
//     ENTERED_CPUS.fetch_add(1, Ordering::SeqCst)
// }
pub fn current_cpu_id() -> u32 {
    match raw_cpuid::CpuId::new().get_feature_info() {
        Some(finfo) => finfo.initial_local_apic_id() as u32,
        None => 0,
    }
}