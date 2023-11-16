// #![cfg_attr(feature = "axstd", no_std)]
// #![cfg_attr(feature = "axstd", no_main)]

// extern crate alloc;
// use alloc::vec::Vec;

// #[cfg(feature = "axstd")]
// use axstd::println;
// const PLASH_START: usize = 0x22000000;
// const RUN_START: usize = 0xffff_ffc0_8010_0000;


// #[cfg_attr(feature = "axstd", no_mangle)]
// fn main() {
//     let num_app_ptr = PLASH_START as usize as *const usize;
//     let num_app = unsafe { num_app_ptr.read_volatile() };
//     let app_start: Vec<usize>  =  unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app * 2).to_vec()};
        
//     println!("Load payload ...");
//     for i in 0..num_app {
//         let start = (app_start[i*2]+PLASH_START) as *const u8;
//         let size = app_start[i*2+1];
//         println!("{} {}", start as usize, size);
//         let code = unsafe { core::slice::from_raw_parts(start, size)};
//         println!("load code {:?}; address [{:?}]", code, code.as_ptr());
//         // app running aspace
//         // SBI(0x80000000) -> App <- Kernel(0x80200000)
//         // 0xffff_ffc0_0000_0000
//         let new_addr = RUN_START as usize + app_start[i*2] - app_start[0];
//         let run_code = unsafe {
//             core::slice::from_raw_parts_mut(new_addr as *mut u8, size)
//         };
//         run_code.copy_from_slice(code);
//         println!("run code {:?}; address [{:?}]", run_code, run_code.as_ptr());
//         println!("Execute app ...");

//         // execute app
//         unsafe { core::arch::asm!("
//             mv      t2, {run_start}
//             jalr    t2",
//             run_start = in(reg) new_addr,
//         )}
//     }

//     println!("Load payload ok!");
// }

#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(asm_const)] 
#[cfg(feature = "axstd")]
use axstd::println;
const PLASH_START: usize = 0x22000000;

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
    let load_start = PLASH_START as *const u8;
    let load_size = 32; // Dangerous!!! We need to get accurate size of apps.
    println!("Load payload ...");
    let load_code = unsafe { core::slice::from_raw_parts(load_start,load_size) };
    println!("load code {:?}; address [{:?}]", load_code, load_code.as_ptr());

    // app running aspace
    // SBI(0x80000000) -> App <- Kernel(0x80200000)
    // 0xffff_ffc0_0000_0000
    const RUN_START: usize = 0xffff_ffc0_8010_0000;
    let run_code = unsafe {
        core::slice::from_raw_parts_mut(RUN_START as *mut u8, load_size)
    };
    run_code.copy_from_slice(load_code);
    println!("run code {:?}; address [{:?}]", run_code, run_code.as_ptr());
    println!("Load payload ok!");
    register_abi(SYS_HELLO, abi_hello as usize);
    register_abi(SYS_PUTCHAR, abi_putchar as usize);
    register_abi(SYS_EXIT, abi_exit as usize);
    println!("Execute app ...");
    // execute app
    let arg0: u8 = b'A';

    // execute app
    unsafe { core::arch::asm!("
        li      t0, {abi_num}
        slli    t0, t0, 3
        la      t1, {abi_table}
        add     t1, t1, t0
        ld      t1, (t1)
        jalr    t1
        li      t2, {run_start}
        jalr    t2
        j       .",
        run_start = const RUN_START,
        abi_table = sym ABI_TABLE,
        //abi_num = const SYS_HELLO,
        abi_num = const SYS_EXIT,
        //in("a0") arg0,
    )}
    unsafe { core::arch::asm!("
        li      t2, {run_start}
        jalr    t2
        j       .",
        run_start = const RUN_START,
    )}
}