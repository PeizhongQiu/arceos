//! Description tables (per-CPU GDT, per-CPU ISS, IDT)

use crate::arch::{GdtStruct, IdtStruct, TaskStateSegment};
use lazy_init::LazyInit;
use x86::{segmentation, segmentation::SegmentSelector, Ring};

static IDT: LazyInit<IdtStruct> = LazyInit::new();

#[percpu::def_percpu]
static TSS: LazyInit<TaskStateSegment> = LazyInit::new();

#[percpu::def_percpu]
static GDT: LazyInit<GdtStruct> = LazyInit::new();

fn init_percpu() {
    unsafe {
        let tss = TSS.current_ref_mut_raw();
        let gdt = GDT.current_ref_mut_raw();
        tss.init_by(TaskStateSegment::new());
        gdt.init_by(GdtStruct::new(tss));
        gdt.load();
        segmentation::load_es(SegmentSelector::from_raw(0));
        segmentation::load_ss(SegmentSelector::from_raw(0));
        segmentation::load_ds(SegmentSelector::from_raw(0));
        IDT.load();
        gdt.load_tss();

        // PAT0: WB, PAT1: WC, PAT2: UC
        x86::msr::wrmsr(x86::msr::IA32_PAT, 0x070106);
    }
}

/// Initializes IDT, GDT on the primary CPU.
pub(super) fn init_primary() {
    axlog::ax_println!("\nInitialize IDT & GDT...");
    IDT.init_by(IdtStruct::new());
    init_percpu();
}

/// Initializes IDT, GDT on secondary CPUs.
#[cfg(feature = "smp")]
pub(super) fn init_secondary() {
    init_percpu();
}
