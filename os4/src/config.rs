pub const USER_STACK_SIZE: usize = 4096;
// pub const KERNEL_STACK_SIZE: usize = 4096 * 20;
// pub const KERNEL_HEAP_SIZE: usize = 0x20000;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;

pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;

pub const MAX_APP_NUM: usize = 16;
pub const APP_BASE_ADDRESS: usize = 0x80400000;
pub const APP_SIZE_LIMIT: usize = 0x20000;
pub const MAX_SYSCALL_NUM: usize = 500;

/// 常数 CLOCK_FREQ 是一个预先获取到的各平台不同的时钟频率，单位为赫兹，也就是一秒钟之内计数器的增量;
/// 说是 CPU 主频，但实际就是 time 寄存器自增的频率而已，这个必须是一个稳定的值，真正 CPU 运行的频率不一定。
pub const CLOCK_FREQ: usize = 0x989680; // 10MHz; see dump.dts -> cpus: timebase-frequency

// in (old) qemu virt machine:
// pub const CLOCK_FREQ: usize = 12500000; // 12.5MHz (old qemu)

pub const MEMORY_END: usize = 0x80800000;

pub const PAGE_SIZE: usize = 1 << 12;
pub const PAGE_SIZE_BITS: usize = 0xc;

pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1; // 0xffff_ffff_ffff_f000
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE; // 0xffff_ffff_ffff_e000

/// Return (bottom, top) of a kernel stack in kernel space.
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}
