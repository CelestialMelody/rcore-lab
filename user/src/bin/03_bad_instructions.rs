#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

#[no_mangle]
pub fn main() -> ! {
    println!("Try to execute privileged instruction in U Mode");
    println!("Kernel should kill this application!");
    unsafe {
        // sret is in S-mode
        core::arch::asm!("sret");
    }
    panic!("FAIL: T.T\n");
}