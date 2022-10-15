mod context;

use crate::batch::run_next_app;
use crate::syscall::syscall;
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Trap},
    stval, stvec,
};

pub use context::TrapContext;

core::arch::global_asm!(include_str!("trap.S"));

/// 在 RV64 中， `stvec` 是一个 64 位的 CSR，在中断使能的情况下，保存了中断处理的入口地址。它有两个字段：
///  - MODE 位于 [1:0]，长度为 2 bits；
///  - BASE 位于 [63:2]，长度为 62 bits。
/// 
/// 当 MODE 字段为 0 的时候， `stvec` 被设置为 Direct 模式，此时进入 S 模式的 Trap 无论原因如何，处理 Trap 的入口地址都是 `BASE<<2` ， CPU 会跳转到这个地方进行异常处理。
pub fn init() {
    extern "C" {
        // 引入外部符号 __alltraps ，并将 stvec 设置为 Direct 模式指向它的地址
        fn __alltraps(); // 从汇编中获取 __alltraps 的地址
    }
    unsafe {
        // stvec 控制 Trap 处理代码的入口地址
        stvec::write(__alltraps as usize, TrapMode::Direct); // 设置中断向量表
    }
}

#[no_mangle]
// 在 __restore 的时候 a0 寄存器在调用 trap_handler 前后并没有发生变化，
// 仍然指向分配 Trap 上下文之后的内核栈栈顶，和此时 sp 的值相同，这里的 sp <- a0 并不会有问题；
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();
    // 根据 scause 寄存器所保存的 Trap 的原因进行分发处理。无需手动操作这些 CSR ，而是使用 Rust 的 riscv 库来更加方便的做这些事情
    match scause.cause() { // 获取中断原因
        Trap::Exception(Exception::UserEnvCall) => { // 触发 Trap 的原因是来自 U 特权级的 Environment Call，也就是系统调用
            // 首先修改保存在内核栈上的 Trap 上下文里面 sepc，让其增加 4。
            // 这是因为, 这是一个由 ecall 指令触发的系统调用，在进入 Trap 的时候，
            // 硬件会将 sepc 设置为这条 ecall 指令所在的地址（因为它是进入 Trap 之前最后一条执行的指令）。
            // 而在 Trap 返回之后，我们希望应用程序控制流从 ecall 的下一条指令开始执行。
            // 因此我们只需修改 Trap 上下文里面的 sepc，让它增加 ecall 指令的码长，也即 4 字节。
            // 这样在 __restore 的时候 sepc 在恢复之后就会指向 ecall 的下一条指令，并在 sret 之后从那里开始执行。
            cx.sepc += 4; // 跳过 ecall 指令

            // 用来保存系统调用返回值的 a0 寄存器也会同样发生变化。
            // 我们从 Trap 上下文取出作为 syscall ID 的 a7(x17) 和系统调用的三个参数 a0~a2 传给 syscall 函数并获取返回值。 
            // syscall 函数是在 syscall 子模块中实现的。 这段代码是处理正常系统调用的控制逻辑。
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize; // 调用系统调用
        }
        // 分别处理应用程序出现访存错误和非法指令错误的情形。
        // 此时需要打印错误信息并调用 run_next_app 直接切换并运行下一个应用程序。
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => { // 写入内存错误
            error!("[kernel] PageFault in application at {:#x}, core dumped.", stval);
            run_next_app();
        }
        Trap::Exception(Exception::IllegalInstruction) => { // 非法指令
            error!("[kernel] IllegalInstruction in application at {:#x}, core dumped.", stval);
            run_next_app();
        }
        _ => {
            panic!(
                "[kernel] Unhandled trap: {:?}, stval: {:#x}, core dumped.",
                scause.cause(),
                stval
            );
        }
    }
    cx
}