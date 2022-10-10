#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

#[no_mangle]
pub fn main() -> ! {
    let mut sstatus: usize;
    println!("Try to access privileged CSR in U Mode");
    println!("Kernel should kill this application!");
    unsafe {
        // csrr and sstatus is a CSR register
        core::arch::asm!("csrr {}, sstatus", out(reg) sstatus);
    }
    panic!("(-_-) I get sstatus:{:x}\nFAIL: T.T\n", sstatus);
}
