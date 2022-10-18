#![allow(unused)]
// use core::arch::asm;

const SBI_SET_TIEMR: usize = 0;
const SBI_CONSOLE_PUTCHAR: usize = 1;
const SBI_CONSOLE_GETCHAR: usize = 2;
const SBI_CLEAR_IPI: usize = 3;
const SBI_SEND_IPI: usize = 4;
const SBI_REMOTE_FENCE_I: usize = 5;
const SBI_REMOTE_SFENCE_VMA: usize = 6;
const SBI_REMOTE_SFENCE_VMA_ASID: usize = 7;
const SBI_SHUTDOWN: usize = 8;

/// which 表示请求 RustSBI 的服务的类型;
/// arg0 ~ arg2 表示传递给 RustSBI 的 3 个参数;
/// RustSBI 在将请求处理完毕后，会给内核一个返回值，这个返回值也会被 sbi_call 函数返回
#[inline(always)]
fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret;
    unsafe {
        core::arch::asm!(
            "li x16, 0",
            "ecall", // ecall 指令会触发 SBI 调用
            inlateout("x10") arg0 => ret, // x10 作为输入参数，返回值保存在 x10 中
            in("x11") arg1,
            in("x12") arg2,
            in("x17") which,
        );
    };
    ret
}

/// 通过调用 sbi_call 函数，将请求发送给 RustSBI，RustSBI 将会将字符 ch 输出到控制台
pub fn console_putchar(ch: usize) {
    sbi_call(SBI_CONSOLE_PUTCHAR, ch, 0, 0);
}

/// 通过调用 sbi_call 函数，将请求发送给 RustSBI，RustSBI 将会从控制台读取一个字符
pub fn console_getchar() -> usize {
    sbi_call(SBI_CONSOLE_GETCHAR, 0, 0, 0)
}

/// 通过调用 sbi_call 函数，将请求发送给 RustSBI，RustSBI 将会将内核停止运行
pub fn shutdown() -> ! {
    sbi_call(SBI_SHUTDOWN, 0, 0, 0);
    panic!("It should shutdown!");
}
