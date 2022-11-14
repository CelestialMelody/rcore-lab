#![no_std]
#![no_main]
#![feature(panic_info_message)]

// use log::*;
#[macro_use]
extern crate log;

#[macro_use]
mod console;
mod lang_items;
mod logging;
mod sbi; // 将内核与 RustSBI 通信的相关功能实现在子模块 sbi 中，加入 mod sbi 将该子模块加入的项目

mod batch;
mod sync; // src 其他文件夹也视为mod 但需要提供mod.rs
mod syscall;
mod trap;

core::arch::global_asm!(include_str!("entry.asm"));
core::arch::global_asm!(include_str!("link_app.S"));

fn clear_bss() {
    extern "C" {
        fn sbss(); // why fn ?
        fn ebss();
    }
    // 尝试从其他地方找到全局符号 sbss 和 ebss ，
    // 它们由链接脚本 linker.ld 给出，
    // 并分别指出需要被清零的 .bss 段的起始和终止地址

    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }

    // (sbss as usize..ebss as usize)
    //         // 遍历该地址区间并逐字节进行清零
    //         .for_each(|a| unsafe {
    //             (a as *mut u8).write_volatile(0)
    //         }
    // )
}
// 通过宏将 rust_main 标记为 #[no_mangle] 以避免编译器对它的名字进行混淆，
// 不然在链接的时候， entry.asm 将找不到 main.rs 提供的外部符号 rust_main 从而导致链接失败
#[no_mangle]
fn rust_main() -> ! {
    extern "C" {
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn sbss();
        fn ebss();
        fn boot_stack();
        fn boot_stack_top();
    }
    clear_bss();
    logging::init();
    println!("[kernel] Hello, world");
    println!("[kernel] Hello, rCore");

    trace!(
        "[kernel] .text [{:#x}, {:#x})",
        stext as usize,
        etext as usize
    );
    debug!(
        "[kernel] .rodata [{:#x}, {:#x})",
        srodata as usize, erodata as usize
    );
    info!(
        "[kernel] .data [{:#x}, {:#x})",
        sdata as usize, edata as usize
    );
    warn!(
        "[kernel] boot stack [{:#x}, {:#x})",
        boot_stack as usize, boot_stack_top as usize
    );
    error!("[kernel] .bss [{:#x}, {:#x})", sbss as usize, ebss as usize);

    trap::init();
    batch::init();
    batch::run_next_app();
}
