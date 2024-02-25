
use core::sync::atomic::{AtomicU32, Ordering};
static ENTERED_CPUS: AtomicU32 = AtomicU32::new(0);

pub fn current_cpu_id() -> u32 {
    match raw_cpuid::CpuId::new().get_feature_info() {
        Some(finfo) => finfo.initial_local_apic_id() as u32,
        None => 0,
    }
}

pub fn thread_pointer() -> usize {
    let ret;
    unsafe { core::arch::asm!("mov {0}, gs:0", out(reg) ret, options(nostack)) }; // PerCpu::self_vaddr
    ret
}

use core::fmt::{Debug, Formatter, Result};

use super::consts::{PER_CPU_ARRAY_PTR, PER_CPU_SIZE};
// pub const PER_CPU_ARRAY_PTR: *mut PerCpu = ekernel as _;
// pub const PER_CPU_SIZE: usize = 512 * 1024; // 512 KB
// use super::header::HvHeader;
pub use memory_addr::{PhysAddr, VirtAddr, PAGE_SIZE_4K};
use hypercraft::{VCpu, HyperResult, LinuxContext};

static ACTIVATED_CPUS: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Eq, PartialEq)]
pub enum CpuState {
    HvDisabled,
    HvEnabled,
}

#[repr(C, align(4096))]
pub struct PerCpu {
    /// Referenced by arch::cpu::thread_pointer() for x86_64.
    self_vaddr: VirtAddr,

    pub id: u32,
    pub state: CpuState,
    // pub vcpu: Vcpu,
    // arch: ArchPerCpu,
    linux: LinuxContext,
    // Stack will be placed here.
}

impl PerCpu {
    // pub fn new<'a>() -> HyperResult<&'a mut Self> {
    //     // if Self::entered_cpus() >= HvHeader::get().max_cpus {
    //     //     return hv_result_err!(EINVAL);
    //     // }

    //     let cpu_id = current_cpu_id();
    //     let vaddr = PER_CPU_ARRAY_PTR as VirtAddr + cpu_id as usize * PER_CPU_SIZE;
    //     let ret = unsafe { &mut *(vaddr as *mut Self) };
    //     ret.id = cpu_id;
    //     ret.self_vaddr = vaddr;
    //     unsafe { x86::msr::wrmsr(x86::msr::IA32_GS_BASE, vaddr as u64) };
    //     Ok(ret)
    // }

    // pub fn current<'a>() -> &'a Self {
    //     Self::current_mut()
    // }

    // pub fn current_mut<'a>() -> &'a mut Self {
    //     unsafe { &mut *(thread_pointer() as *mut Self) }
    // }

    // pub fn stack_top(&self) -> VirtAddr {
    //     self as *const _ as VirtAddr + PER_CPU_SIZE - 8
    // }

    // pub fn entered_cpus() -> u32 {
    //     ENTERED_CPUS.load(Ordering::Acquire)
    // }

    // pub fn activated_cpus() -> u32 {
    //     ACTIVATED_CPUS.load(Ordering::Acquire)
    // }

    // pub fn init(&mut self, linux_sp: usize) -> HyperResult {
    //     info!("CPU {} init...", self.id);

    //     // Save CPU state used for linux.
    //     self.state = CpuState::HvDisabled;
    //     self.linux = LinuxContext::load_from(linux_sp);
    //     // self.arch.init();

    //     // Activate hypervisor page table on each cpu.
    //     unsafe { crate::memory::hv_page_table().read().activate() };

    //     // Initialize vCPU. Use `ptr::write()` to avoid dropping
    //     // unsafe { core::ptr::write(&mut self.vcpu, Vcpu::new(&self.linux, npt)?) };

    //     // self.state = CpuState::HvEnabled;
    //     Ok(())
    // }

    // pub fn activate_vmm(&mut self) -> HyperResult {
    //     println!("Activating hypervisor on CPU {}...", self.id);
    //     ACTIVATED_CPUS.fetch_add(1, Ordering::SeqCst);

    //     self.vcpu.enter(&self.linux)?;
    //     unreachable!()
    // }

    // pub fn deactivate_vmm(&mut self, ret_code: usize) -> HyperResult {
    //     println!("Deactivating hypervisor on CPU {}...", self.id);
    //     ACTIVATED_CPUS.fetch_sub(1, Ordering::SeqCst);

    //     self.vcpu.set_return_val(ret_code);
    //     self.vcpu.exit(&mut self.linux)?;
    //     self.linux.restore();
    //     self.state = CpuState::HvDisabled;
    //     self.linux.return_to_linux(self.vcpu.regs());
    // }

    // pub fn fault(&mut self) -> HyperResult {
    //     warn!("VCPU fault: {:#x?}", self);
    //     self.vcpu.inject_fault()?;
    //     Ok(())
    // }
}

impl Debug for PerCpu {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut res = f.debug_struct("PerCpu");
        res.field("id", &self.id)
            .field("self_vaddr", &self.self_vaddr)
            .field("state", &self.state);
        // if self.state != CpuState::HvDisabled {
        //     res.field("vcpu", &self.vcpu);
        // } else {
        //     res.field("linux", &self.linux);
        // }
        res.field("linux", &self.linux);
        res.finish()
    }
}
