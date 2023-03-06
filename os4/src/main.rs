#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

#[macro_use]
mod console;

#[macro_use]
extern crate log;

#[macro_use]
extern crate bitflags;

// `alloc` with `#![no_std]` support, see [`link`](https://doc.rust-lang.org/edition-guide/rust-2018/path-changes.html#an-exception-for-extern-crate)
extern crate alloc;

mod config;
mod lang_items;
mod loader;
mod logging;
mod mm;
mod sbi;
mod sync;
mod syscall;
mod task;
mod timer;
mod trap;

core::arch::global_asm!(include_str!("entry.asm"));
core::arch::global_asm!(include_str!("link_app.S"));

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    // 找到全局符号 sbss 和 ebss ，
    // 它们由链接脚本 linker.ld 给出，需要被清零

    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}
// 链接的时候 entry.asm 寻找符号 rust_main
// 将 rust_main 标记为 #[no_mangle] 以避免编译器对它的名字进行混淆
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

    mm::init();
    println!("[kernel] back to world!");
    mm::remap_test();

    trap::init();
    // loader::load_apps();

    // 为了避免 S 特权级时钟中断被屏蔽，需要在内核态下开启时钟中断
    // 设置了 sie.stie 使得 S 特权级时钟中断不会被屏蔽
    trap::enable_timer_interrupt();
    // 设置第一个 10ms 的计时器
    timer::set_next_trigger();

    task::run_first_task();

    panic!("Unreachable in rust_main!");
}
