
use core::sync::atomic::{AtomicU32, Ordering};

static ENTERED_CPUS: AtomicU32 = AtomicU32::new(0);

extern "C" {
    fn __core_end();
}
/// Size of the per-CPU data (stack and other CPU-local data).
pub const PER_CPU_SIZE: usize = 512 * 1024; // 512 KB

unsafe extern "sysv64" fn switch_stack(linux_sp: usize) -> i32 {
    let linux_tp = x86::msr::rdmsr(x86::msr::IA32_GS_BASE) as u64;
    let cpu_id = ENTERED_CPUS.fetch_add(1, Ordering::SeqCst);
    let per_cpu_array_ptr: usize = __core_end as usize;
    let vaddr = per_cpu_array_ptr + cpu_id as usize * PER_CPU_SIZE;
    // if cpu_id >= HvHeader::get().max_cpus {
    //     error!("cpuid({}) is bigger than max_cpus!!!", cpu_id);
    //     return -1;
    // }
    // let hv_sp = cpu_data.stack_top();
    let hv_sp = vaddr + PER_CPU_SIZE - 8;
    let ret;
    core::arch::asm!("
        mov [rsi], {linux_tp}   // save gs_base to stack
        mov rcx, rsp
        mov rsp, {hv_sp}
        push rcx
        call {entry}
        pop rsp",
        entry = sym super::rust_entry_hv,
        linux_tp = in(reg) linux_tp,
        hv_sp = in(reg) hv_sp,
        in("rdi") cpu_id,
        in("rsi") linux_sp,
        lateout("rax") ret,
        out("rcx") _,
    );
    x86::msr::wrmsr(x86::msr::IA32_GS_BASE, linux_tp);
    ret
}

#[naked]
#[no_mangle]
#[link_section = ".text.boot"]
pub unsafe extern "C" fn _start() -> i32 {
    core::arch::asm!("
        // rip is pushed
        cli
        push rbp
        push rbx
        push r12
        push r13
        push r14
        push r15
        push 0  // skip gs_base

        mov rdi, rsp
        call {0}

        pop r15 // skip gs_base
        pop r15
        pop r14
        pop r13
        pop r12
        pop rbx
        pop rbp
        ret
        // rip will pop when return",
        sym switch_stack,
        options(noreturn),
    );
}
