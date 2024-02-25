#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate libax;

#[cfg(not(target_arch = "aarch64"))]
use libax::{
    hv::{
        self, GuestPageTable, GuestPageTableTrait, HyperCallMsg, HyperCraftHalImpl, PerCpu, Result,
        VCpu, VmCpus, VmExitInfo, VM, phys_to_virt, LinuxContext
    },
    info,
};

use page_table_entry::MappingFlags;

#[cfg(target_arch = "x86_64")]
use device::{X64VcpuDevices, X64VmDevices};

#[cfg(target_arch = "x86_64")]
#[cfg(target_arch = "x86_64")]
#[path = "device/x86_64/mod.rs"]
mod device;

extern "C" {
    fn ekernel();
}
/// Size of the per-CPU data (stack and other CPU-local data).
pub const PER_CPU_SIZE: u64 = 512 * 1024; // 512 KB

#[no_mangle]
fn main(cpu_id: usize, npt: usize, linux: &LinuxContext) {
    
    println!("Hello, hv!");

    #[cfg(target_arch = "x86_64")]
    {
        info!("{:x?}",linux);
        println!("into main {}", cpu_id);

        let mut p = PerCpu::<HyperCraftHalImpl>::new(cpu_id);
        info!("PerCpu");
        p.hardware_enable().unwrap();
        info!("hardware_enable");

        let mut vcpus = VmCpus::<HyperCraftHalImpl, X64VcpuDevices<HyperCraftHalImpl>>::new();
        info!("new vcpus");
        let per_cpu_array_ptr: u64 = ekernel as u64 + cpu_id as u64 * PER_CPU_SIZE;
        let hv_sp = per_cpu_array_ptr + PER_CPU_SIZE - 8;
        vcpus.add_vcpu(VCpu::new_linux_sp(0, p.vmcs_revision_id(), &linux, hv_sp, npt).unwrap());
        info!("add_vcpu");
        let mut vm = VM::<HyperCraftHalImpl, X64VcpuDevices<HyperCraftHalImpl>, X64VmDevices<HyperCraftHalImpl>>::new(vcpus);
        info!("new vm");
        vm.bind_vcpu(0);
        info!("bind_vcpu");
        // if cpu_id == 0 {
        //     let (_, dev) = vm.get_vcpu_and_device(0).unwrap();
        //     *(dev.console.lock().backend()) = device::device_emu::MultiplexConsoleBackend::Primary;

        //     // for v in 0..256 {
        //     //     libax::hv::set_host_irq_enabled(v, true);
        //     // }
        // }

        println!("Running guest...");
        println!("{:?}", vm.run_vcpu(0));

        p.hardware_disable().unwrap();

        panic!("done");

        return;
    }
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub fn main_secondary(hart_id: usize) {
    println!("secondary into main {}", hart_id);

    // main(1);

    /*
    loop {
        libax::thread::sleep(libax::time::Duration::from_secs(5));
        println!("secondary tick");
    } */
}
