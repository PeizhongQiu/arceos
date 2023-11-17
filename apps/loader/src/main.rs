#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(asm_const)]

extern crate alloc;
use alloc::vec::Vec;

mod page_table;
mod appmanager;
use page_table::PageTable;
use appmanager::AppManager;
#[cfg(feature = "axstd")]
use axstd::println;
const PLASH_START: usize = 0x22000000;
const RUN_START: usize = 0x4010_0000;

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

#[link_section = ".data.app_page_table"]
static mut APP_PAGE_TABLE: [PageTable; 10] = [PageTable::new(0), PageTable::new(1), PageTable::new(2), PageTable::new(3),PageTable::new(4), PageTable::new(5),
                    PageTable::new(6), PageTable::new(7),PageTable::new(8), PageTable::new(9),];

static mut APP_MANAGER: AppManager = AppManager::new();

fn load_and_run(id:usize) {
    println!("Load first payload ...");
    let start = (unsafe { APP_MANAGER.get_app_start(id*2) }+PLASH_START) as *const u8;
    let size = unsafe { APP_MANAGER.get_app_start(id*2+1) };
    println!("{} {}", start as usize, size);
    let code = unsafe { core::slice::from_raw_parts(start, size)};
    println!("load code {:?}; address [{:?}]", code, code.as_ptr());
        
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
    
    unsafe { core::arch::asm!("
        la      a0, {abi_table}
        li      t2, {run_start}
        jalr    t2",
        run_start = const RUN_START,
        abi_table = sym ABI_TABLE,
    )}

}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {

    unsafe { switch_app_aspace(APP_PAGE_TABLE[0].clone()); }

    let num_app_ptr = PLASH_START as usize as *const usize;
    unsafe { APP_MANAGER.update_num_app(num_app_ptr.read_volatile()) };
    let app_start  =  unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), APP_MANAGER.get_num_app() * 2).to_vec()};
    unsafe { APP_MANAGER.update_app_start(app_start) };
        
    load_and_run(0);
    unsafe { switch_app_aspace(APP_PAGE_TABLE[1].clone()); }

    load_and_run(1);
    println!("Load payload ok!");
}


unsafe fn switch_app_aspace(to:PageTable) {
    use riscv::register::satp;
    let page_table_root = to.APP_PT_SV39.as_ptr() as usize - axconfig::PHYS_VIRT_OFFSET;
    satp::set(satp::Mode::Sv39, 0, page_table_root >> 12);
    riscv::asm::sfence_vma_all();
}