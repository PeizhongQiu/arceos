#![feature(asm_const)]
#![no_std]
#![no_main]
#![feature(linkage)]

const SYS_HELLO: usize = 1;
const SYS_PUTCHAR: usize = 2;
const SYS_EXIT:usize = 3;
static mut ENTRY: usize = 0;

fn print_hello() {
    unsafe{
        core::arch::asm!("
            li      t0, {abi_num}
            slli    t0, t0, 3
            mv      a7, {abi_table}
            add     t1, a7, t0
            ld      t1, (t1)
            jalr    t1",
            abi_num = const SYS_HELLO,
            abi_table = in(reg) ENTRY,
            clobber_abi("C"),
        )
    }
    
}
fn putchar(c : char) {
    let arg0:u8 = c as u8;
    unsafe{
        core::arch::asm!("
            li      t0, {abi_num}
            slli    t0, t0, 3
            mv      a7, {abi_table}
            add     t1, a7, t0
            ld      t1, (t1)
            jalr    t1",
            abi_num = const SYS_PUTCHAR,
            abi_table = in(reg) ENTRY,
            in("a0") arg0,
            clobber_abi("C"),
        )
    }
    
}

unsafe fn puts(s : &str) {
    for (_i, item) in s.chars().enumerate() {
        putchar(item as char);
    }
}

fn exit() {
    unsafe{
        core::arch::asm!("
            li      t0, {abi_num}
            slli    t0, t0, 3
            mv      a7, {abi_table}
            add     t1, a7, t0
            ld      t1, (t1)
            jalr    t1",
            abi_num = const SYS_EXIT,
            abi_table = in(reg) ENTRY,
            clobber_abi("C"),
        )
    }
    
}

#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start(entry: usize) {
    ENTRY = entry;
    print_hello();
    puts("exercise 5 puts OK!!!!");
    // assert!(ENTRY==entry);
    exit();
    
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
