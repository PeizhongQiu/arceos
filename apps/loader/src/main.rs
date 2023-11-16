#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

extern crate alloc;
use alloc::vec::Vec;

#[cfg(feature = "axstd")]
use axstd::println;
const PLASH_START: usize = 0x22000000;


#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let num_app_ptr = PLASH_START as usize as *const usize;
    let num_app = unsafe { num_app_ptr.read_volatile() };
    let mut app_start: Vec<usize>  =  unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app * 2).to_vec()};
        
    println!("Load payload ...");
    for i in 0..num_app {
        let start = (app_start[i*2]+PLASH_START) as *const u8;
        let size = app_start[i*2+1];
        println!("{} {}", start as usize, size);
        let code = unsafe { core::slice::from_raw_parts(start, size)};
        println!("content: {:?}: ", code);
    }

    println!("Load payload ok!");
}