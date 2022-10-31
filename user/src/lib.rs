#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

#[macro_use]
pub mod console;
mod lang_items;
mod syscall;

/// `alloc` with `#![no_std]` support, see [`link`](https://doc.rust-lang.org/edition-guide/rust-2018/path-changes.html#an-exception-for-extern-crate)
// extern crate alloc;
use syscall::*;

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

fn clear_bss() {
    extern "C" {
        fn start_bss();
        fn end_bss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(
            // from_raw_parts_mut() returns a mutable slice of memory
            start_bss as usize as *mut u8,
            end_bss as usize - start_bss as usize,
        )
        .fill(0);
    }
}

#[no_mangle]
// 使用 Rust 的宏将 _start 这段代码编译后的汇编代码中放在一个名为 .text.entry 的代码段中
// 方便在后续链接的时候调整它的位置使得它能够作为用户库的入口
#[link_section = ".text.entry"] // .text.entry is a section name
pub extern "C" fn _start() -> ! {
    // 手动清空需要零初始化的 .bss 段
    // 很遗憾到目前为止底层的批处理系统还没有这个能力，所以我们只能在用户库中完成
    clear_bss();
    // 然后调用 main 函数得到一个类型为 i32 的返回值
    // 最后调用用户库提供的 exit 接口退出应用程序，并将 main 函数的返回值告知批处理系统
    exit(main());
    // panic!("unreachable after sys_exit!");
}

// 使用 Rust 的宏将其函数符号 main 标志为弱链接
// 这样在最后链接的时候，虽然在 lib.rs 和 bin 目录下的某个应用程序都有 main 符号，
// 但由于 lib.rs 中的 main 符号是弱链接，链接器会使用 bin 目录下的应用主逻辑作为 main
// 这里我们主要是进行某种程度上的保护，如果在 bin 目录下找不到任何 main ，那么编译也能够通过，但会在运行时报错
// UNKWON: 用户库的main返回值并非是符合exits参数的
// 1. 用户的main与这里的main有什么关系吗？
// 2. 用户的main退出时与这里的exit有什么关系吗？
#[linkage = "weak"] // -> add #![feature(linkage)]
#[no_mangle]
fn main() -> i32 {
    panic!("main() not defined in apps!");
}

pub fn exit(code: i32) -> ! {
    sys_exit(code);
}

pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    sys_read(fd, buf)
}

pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}
