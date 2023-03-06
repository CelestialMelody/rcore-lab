//! Constants used in rCore

pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;

pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;

/// 物理页帧右边界，即内存最大物理地址
pub const MEMORY_END: usize = 0x80800000;

pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc;

pub const MAX_SYSCALL_NUM: usize = 500;
pub const MAX_APP_NUM: usize = 16;

/// TRAMPOLINE is the address of the trampoline page, which is used to store the trap context.
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;

/// 常数 CLOCK_FREQ 是一个预先获取到的各平台不同的时钟频率，单位为赫兹，也就是一秒钟之内计数器的增量;
/// 说是 CPU 主频，但实际就是 time 寄存器自增的频率而已，这个必须是一个稳定的值，真正 CPU 运行的频率不一定。
pub const CLOCK_FREQ: usize = 12500000;
// in (old) qemu virt machine:
// pub const CLOCK_FREQ: usize = 12500000; // 12.5MHz (old qemu)

/// Return (bottom, top) of a kernel stack in kernel space.
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}
