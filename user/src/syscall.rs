#![allow(unused)]
pub const SYSCALL_READ: usize = 63;
pub const SYSCALL_WRITE: usize = 64;
pub const SYSCALL_EXIT: usize = 93;
pub const SYSCALL_YIELD: usize = 124;
pub const SYSCALL_GET_TIME: usize = 169;
pub const SYSCALL_TASK_INFO: usize = 410;

use super::TimeVal;
use crate::TaskInfo;

/// 所有的系统调用都封装成 syscall 函数，支持传入 syscall ID 和 3 个参数
pub fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        // 相比 global_asm! ， asm! 宏可以获取上下文中的变量信息并允许嵌入的汇编代码对这些变量进行操作
        core::arch::asm!( // asm! 宏可以将汇编代码嵌入到局部的函数上下文中
            "ecall", // `ecall` 指令触发 Trap
            // a0 寄存器，它同时作为输入和输出，
            // 因此我们将 in 改成 inlateout ，并在行末的变量部分使用 {in_var} => {out_var} 的格式，
            // 其中 {in_var} 和 {out_var} 分别表示上下文中的输入变量和输出变量
            inlateout ("x10") args[0] => ret, // `a0` 保存系统调用的返回值
            // 输入参数 args[1] 绑定到 ecall 的输入寄存器 x11 即 a1 中，
            // 编译器自动插入相关指令并保证在 ecall 指令被执行之前寄存器 a1 的值与 args[1] 相同
            in ("x11") args[1], //  `a0~a6` 保存系统调用的参数
            in ("x12") args[2],
            in ("x17") id, // `a7` 用来传递 syscall ID
        );
    }
    ret
}

/// 功能：将内存中缓冲区中的数据写入文件。
/// 参数：`fd` 表示待写入文件的文件描述符；
///      `buf` 表示内存中缓冲区的地址段；
/// 返回值：返回成功写入的长度。
/// syscall ID：64
pub fn sys_write(fd: usize, buf: &[u8]) -> isize {
    // sys_write 使用一个 &[u8] 切片类型来描述缓冲区，
    // 这是一个 胖指针 (Fat Pointer)，里面既包含缓冲区的起始地址，还包含缓冲区的长度
    // 可以分别通过 as_ptr 和 len 方法取出它们并独立地作为实际的系统调用参数
    syscall(SYSCALL_WRITE, [fd, buf.as_ptr() as usize, buf.len()])
}

/// 功能：从文件中读取数据到内存缓冲区。
/// 参数：`fd` 表示待读取文件的文件描述符；
///     `buf` 表示内存中缓冲区的地址段；
/// 返回值：返回成功读取的长度。
/// syscall ID：63
pub fn sys_read(fd: usize, buf: &mut [u8]) -> isize {
    syscall(SYSCALL_READ, [fd, buf.as_mut_ptr() as usize, buf.len()])
}

/// 功能：退出应用程序并将返回值告知批处理系统。
/// 参数：`xstate` 表示应用程序的返回值。
/// 返回值：该系统调用不应该返回。
/// syscall ID：93
pub fn sys_exit(xstate: i32) -> ! {
    syscall(SYSCALL_EXIT, [xstate as usize, 0, 0]);
    panic!("sys_exit never returns");
}

pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0])
}

pub fn sys_get_time(time: &TimeVal, tz: usize) -> isize {
    syscall(SYSCALL_GET_TIME, [time as *const _ as usize, tz, 0])
}

pub fn sys_task_info(info: &TaskInfo) -> isize {
    syscall(SYSCALL_TASK_INFO, [info as *const _ as usize, 0, 0])
}
