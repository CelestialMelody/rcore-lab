#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> u32 {
    println!("Hello, user mode!");
    0
}
