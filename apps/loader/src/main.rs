#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(asm_const)]

extern crate alloc;
use alloc::vec::Vec;

#[cfg(feature = "axstd")]
use axstd::println;
const PLASH_START: usize = 0x22000000;
const RUN_START: usize = 0xffff_ffc0_8010_0000;

const SYS_HELLO: usize = 1;
const SYS_PUTCHAR: usize = 2;
const SYS_EXIT: usize = 3;

static mut ABI_TABLE: [usize; 16] = [0; 16];

fn register_abi(num: usize, handle: usize) {
    unsafe { ABI_TABLE[num] = handle; }
}

fn abi_hello() {
    println!("[ABI:Hello] Hello, Apps!");
}

fn abi_putchar(c: char) {
    println!("[ABI:Print] {c}");
}

fn abi_exit() {
    axhal::misc::terminate();
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let num_app_ptr = PLASH_START as usize as *const usize;
    let num_app = unsafe { num_app_ptr.read_volatile() };
    let app_start: Vec<usize>  =  unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app * 2).to_vec()};
        
    println!("Load payload ...");
    for i in 0..num_app {
        let start = (app_start[i*2]+PLASH_START) as *const u8;
        let size = app_start[i*2+1];
        println!("{} {}", start as usize, size);
        let code = unsafe { core::slice::from_raw_parts(start, size)};
        println!("load code {:?}; address [{:?}]", code, code.as_ptr());
        // app running aspace
        // SBI(0x80000000) -> App <- Kernel(0x80200000)
        // 0xffff_ffc0_0000_0000
        let run_code = unsafe {
            core::slice::from_raw_parts_mut(RUN_START as *mut u8, size)
        };
        run_code.copy_from_slice(code);
        println!("run code {:?}; address [{:?}]", run_code, run_code.as_ptr());
        println!("Execute app ...");
        register_abi(SYS_HELLO, abi_hello as usize);
        register_abi(SYS_PUTCHAR, abi_putchar as usize);
        register_abi(SYS_EXIT, abi_exit as usize);
        // execute app

        unsafe {println!("{:x?}",ABI_TABLE);}
        unsafe { 
            let mut addr:usize = 0;
            core::arch::asm!("
            la      {ar}, {abi_table}",
            ar = out(reg) addr,
            abi_table = sym ABI_TABLE,
            );
            println!("{:x?}",addr);
        }   
        
        unsafe { core::arch::asm!("
            la      a0, {abi_table}
            la      t2, {run_start}
            jalr    t2",
            run_start = const RUN_START,
            abi_table = sym ABI_TABLE,
        )}   
    }
    println!("Load payload ok!");
}

