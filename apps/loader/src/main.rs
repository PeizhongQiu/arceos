#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
use core::arch::global_asm;

#[cfg(feature = "axstd")]
use axstd::println;
const PLASH_START: usize = 0x22000000;
global_asm!(include_str!("link_app.S"));

struct AppManager {
    app_num: usize,
    app_start: usize,
    app_end: usize,
}
#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    extern "C" { fn _num_app(); }
    let num_app_ptr = _num_app as usize as *const usize;
    // let num_app = unsafe { num_app_ptr.read_volatile() };
    let mut app_start: [usize; 3] = [0; 3];
    let app_start_raw: &[usize] =  unsafe { core::slice::from_raw_parts(num_app_ptr, 3)};
        
    app_start[..=2].copy_from_slice(app_start_raw);
    let apps_start = app_start[1] as *const u8;
    let apps_size = app_start[2] - app_start[1];
    println!("{:x} {:x} {:x}", app_start[0], app_start[1], apps_size);
    println!("Load payload ...");
    let code = unsafe { core::slice::from_raw_parts(apps_start, apps_size)};
    println!("content: {:?}: ", code);
    println!("Load payload ok!");
}