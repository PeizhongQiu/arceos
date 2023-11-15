#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
use core::arch::global_asm;
extern crate alloc;
use alloc::vec::Vec;

#[cfg(feature = "axstd")]
use axstd::println;
const PLASH_START: usize = 0x22000000;
global_asm!(include_str!("link_app.S"));

struct AppManager {
    app_num: usize,
    app_start: Vec<usize>,
}
#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    extern "C" { fn _num_app(); }
    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = unsafe { num_app_ptr.read_volatile() };
    let mut app_start: Vec<usize>  =  unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1).to_vec()};
        

    let app_manager = AppManager {
        app_num: num_app,
        app_start: app_start,
    };

    let app0_start = app_manager.app_start[0] as *const u8;
    let app0_size = app_manager.app_start[1] - app_manager.app_start[0];
    let app1_start = app_manager.app_start[1] as *const u8;
    let app1_size = app_manager.app_start[2] - app_manager.app_start[1];
    println!("{} {} {} {}", app0_start as usize, app0_size, app1_start as usize, app1_size);
    println!("Load payload ...");
    let code = unsafe { core::slice::from_raw_parts(app0_start, app0_size)};
    let code1 = unsafe { core::slice::from_raw_parts(app1_start, app1_size)};
    println!("content: {:?}: ", code);
    println!("content: {:?}: ", code1);
    println!("Load payload ok!");
}