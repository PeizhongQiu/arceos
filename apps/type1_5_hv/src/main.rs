#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate libax;

#[cfg(not(target_arch = "aarch64"))]
use libax::{
    hv::{
        self, GuestPageTable, GuestPageTableTrait, HyperCallMsg, HyperCraftHalImpl, PerCpu, Result,
        VCpu, VmCpus, VmExitInfo, VM, phys_to_virt,
    },
    info,
};

use page_table_entry::MappingFlags;

#[cfg(target_arch = "x86_64")]
use device::{X64VcpuDevices, X64VmDevices};


#[cfg(target_arch = "x86_64")]
mod x64;

#[cfg(target_arch = "x86_64")]
#[path = "device/x86_64/mod.rs"]
mod device;

#[no_mangle]
fn main(hart_id: usize) {
    println!("Hello, hv!");

    #[cfg(target_arch = "x86_64")]
    {
        println!("into main {}", hart_id);

        let mut p = PerCpu::<HyperCraftHalImpl>::new(hart_id);
        info!("PerCpu");
        p.hardware_enable().unwrap();
        info!("hardware_enable");
        let gpm = x64::setup_gpm(hart_id).unwrap();
        info!("gpm");
        let npt = gpm.nest_page_table_root();
        info!("{:#x?}", gpm);

        let mut vcpus = VmCpus::<HyperCraftHalImpl, X64VcpuDevices<HyperCraftHalImpl>>::new();
        info!("new vcpus");
        vcpus.add_vcpu(VCpu::new(0, p.vmcs_revision_id(), 0x7c00, npt).unwrap());
        info!("add_vcpu");
        let mut vm = VM::<HyperCraftHalImpl, X64VcpuDevices<HyperCraftHalImpl>, X64VmDevices<HyperCraftHalImpl>>::new(vcpus);
        info!("new vm");
        vm.bind_vcpu(0);
        info!("bind_vcpu");
        // if hart_id == 0 {
        //     let (_, dev) = vm.get_vcpu_and_device(0).unwrap();
        //     *(dev.console.lock().backend()) = device::device_emu::MultiplexConsoleBackend::Primary;

        //     for v in 0..256 {
        //         libax::hv::set_host_irq_enabled(v, true);
        //     }
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
