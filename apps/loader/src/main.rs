#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

extern crate alloc;
use alloc::vec::Vec;

#[cfg(feature = "axstd")]
use axstd::println;
const PLASH_START: usize = 0x22000000;
const RUN_START: usize = 0xffff_ffc0_8010_0000;


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
        let new_addr = RUN_START as usize + app_start[i*2] - app_start[0];
        let run_code = unsafe {
            core::slice::from_raw_parts_mut(new_addr as *mut u8, size)
        };
        run_code.copy_from_slice(code);
        println!("run code {:?}; address [{:?}]", run_code, run_code.as_ptr());
        println!("Execute app ...");

        // execute app
        unsafe { core::arch::asm!("
            mv      t2, {run_start}
            jalr    t2",
            run_start = in(reg) new_addr,
        )}
    }

    println!("Load payload ok!");
}