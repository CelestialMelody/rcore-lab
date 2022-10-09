#![no_std]
#![no_main]
#![feature(panic_info_message)]

use log::*;
#[macro_use]
mod console;
mod lang_items;
mod sbi; // 将内核与 RustSBI 通信的相关功能实现在子模块 sbi 中，加入 mod sbi 将该子模块加入的项目
mod logging;


core::arch::global_asm!(include_str!("entry.asm"));

fn clear_bss() {
    extern "C" { // extern “C” 可以引用一个外部的 C 函数接口（这意味着调用它的时候要遵从目标平台的 C 语言调用规范）
        fn sbss();
        fn ebss();
    }
    // 尝试从其他地方找到全局符号 sbss 和 ebss ，
    // 它们由链接脚本 linker.ld 给出，
    // 并分别指出需要被清零的 .bss 段的起始和终止地址 
    (sbss as usize..ebss as usize) // 引用位置标志并将其转成 usize 获取它的地址
            // 遍历该地址区间并逐字节进行清零
            .for_each(|a| unsafe {
                (a as *mut u8).write_volatile(0)
            }
    )
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
    println!("Hello, World");
    println!("Hello, rCore");

    trace!(".text [{:#x}, {:#x})",
        stext as usize,
        etext as usize
        );
    debug!(
        ".rodata [{:#x}, {:#x})",
        srodata as usize,
        erodata as usize
    );
    info!(
        ".data [{:#x}, {:#x})",
        sdata as usize,
        edata as usize
    );
    warn!(
        "boot stack [{:#x}, {:#x})",
        boot_stack as usize,
        boot_stack_top as usize
    );
    error!(
        ".bss [{:#x}, {:#x})",
        sbss as usize,
        ebss as usize
    );

    panic!("Shutdown machine!");
}