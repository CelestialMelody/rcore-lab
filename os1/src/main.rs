#![no_std]
#![no_main]

mod lang_items;

core::arch::global_asm!(include_str!("entry.asm"));

fn main() {
    // println!("Hello, world!");
}
