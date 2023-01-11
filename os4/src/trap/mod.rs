mod context;

use crate::config::{TRAMPOLINE, TRAP_CONTEXT};
use crate::syscall::syscall;
use crate::task::{
    current_trap_cx, current_user_token, exit_current_and_run_next, suspend_current_and_run_next,
};

use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec,
};

pub use context::TrapContext;

core::arch::global_asm!(include_str!("trap.S"));

// stvec 控制 Trap 处理代码的入口地址
// 在 RV64 中， `stvec` 是一个 64 位的 CSR，在中断使能的情况下，保存了中断处理的入口地址。它有两个字段：
//  - MODE 位于 [1:0]，长度为 2 bits；
//  - BASE 位于 [63:2]，长度为 62 bits。
//
// 当 MODE 字段为 0 的时候， `stvec` 被设置为 Direct 模式，此时进入 S 模式的 Trap 无论原因如何，处理 Trap 的入口地址都是 `BASE<<2` ， CPU 会跳转到这个地方进行异常处理。

#[no_mangle]
/// 这就是说，一旦进入内核后再次触发到 S态 Trap，则硬件在设置一些 CSR 寄存器之后，会跳过对通用寄存器的保存过程，
/// 直接跳转到 trap_from_kernel 函数，在这里直接 panic 退出。
/// 这是因为内核和应用的地址空间分离之后，U态 –> S态 与 S态 –> S态 的 Trap 上下文保存与恢复实现方式/Trap 处理逻辑有很大差别。
/// 这里为了简单起见，弱化了 S态 –> S态的 Trap 处理过程：直接 panic 。
pub fn trap_from_kernel() -> ! {
    panic!("a trap from kernel!");
}

pub fn init() {
    set_kernel_trap_entry();
}

// 调用 set_kernel_trap_entry 将 stvec 修改为同模块下另一个函数 trap_from_kernel 的地址。
fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
    }
}

// 让应用 Trap 到 S 的时候可以跳转到 __alltraps
fn set_user_trap_entry() {
    unsafe {
        // 把 stvec 设置为内核和应用地址空间共享的跳板页面的起始地址 (__alltraps) TRAMPOLINE 而不是编译器在链接时看到的 __alltraps 的地址。
        // 这是因为启用分页模式之后，内核只能通过跳板页面上的虚拟地址来实际取得 __alltraps 和 __restore 的汇编代码。
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
}
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer(); // 设置时钟中断
    }
}

#[no_mangle]
pub fn trap_handler() -> ! {
    // 调用 set_kernel_trap_entry 将 stvec 修改为同模块下另一个函数 trap_from_kernel 的地址
    set_kernel_trap_entry();

    // 由于应用的 Trap 上下文不在内核地址空间，因此我们调用 current_trap_cx 来获取当前应用的 Trap 上下文的可变引用而不是像之前那样作为参数传入 trap_handler
    let cx = current_trap_cx();

    let scause = scause::read();
    let stval = stval::read();

    // 根据 scause 寄存器所保存的 Trap 的原因进行分发处理。无需手动操作这些 CSR ，而是使用 Rust 的 riscv 库来更加方便的做这些事情
    match scause.cause() {
        // 获取中断原因
        Trap::Exception(Exception::UserEnvCall) => {
            // 触发 Trap 的原因是来自 U 特权级的 Environment Call，也就是系统调用
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
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
            // x10(a0) 保存返回值; 这里修改的是用户态，应用程序上下文，a0 作为返回值
        }
        // 分别处理应用程序出现访存错误和非法指令错误的情形。
        // 此时需要打印错误信息并调用 run_next_app 直接切换并运行下一个应用程序。
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            // 写入内存错误
            error!(
                "[kernel] PageFault in application, bad addr = {:#x}, bad instruction = {:#x}, core dumped.",
                stval, cx.sepc
            );
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            // 非法指令
            error!(
                "[kernel] IllegalInstruction in application, bad addr = {:#x}, core dumped.",
                stval
            );
            exit_current_and_run_next();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            // 时钟中断
            // 在 trap_handler 函数下新增一个条件分支跳转，当发现触发了一个 S 特权级时钟中断的时候，
            // 首先重新设置一个 10ms 的计时器，
            // 然后调用 suspend_current_and_run_next 函数暂停当前应用并切换到下一个。
            set_next_trigger();
            suspend_current_and_run_next();
        }
        _ => {
            panic!(
                "[kernel] Unhandled trap: {:?}, stval: {:#x}, core dumped.",
                scause.cause(),
                stval
            );
        }
    }

    // 在 trap_handler 完成 Trap 处理之后，我们需要调用 trap_return 返回用户态
    trap_return()
}

#[no_mangle]
// 返回用户态
pub fn trap_return() -> ! {
    set_user_trap_entry();

    // 准备好 __restore 需要两个参数：分别是 Trap 上下文在应用地址空间中的虚拟地址和要继续执行的应用地址空间的 token 。
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = current_user_token();

    // __alltraps 和 __restore 都是指编译器在链接时看到的内核内存布局中的地址
    extern "C" {
        fn __alltraps();
        fn __restore();
    }

    // 跳转到 __restore ，以执行：切换到应用地址空间、从 Trap 上下文中恢复通用寄存器、 sret 继续执行应用。
    // 它的关键在于如何找到 __restore 在内核/应用地址空间中共同的虚拟地址。

    // 计算 __restore 虚地址的过程：
    // 由于 __alltraps 是对齐到地址空间跳板页面的起始地址 TRAMPOLINE 上的，
    // 则 __restore 的虚拟地址只需在 TRAMPOLINE 基础上加上 __restore 相对于 __alltraps 的偏移量即可。
    // 这里 __alltraps 和 __restore 都是指编译器在链接时看到的内核内存布局中的地址。
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;

    unsafe {
        // https://doc.rust-lang.org/nightly/reference/inline-assembly.html

        // 首先需要使用 fence.i 指令清空指令缓存 i-cache 。
        // 这是因为，在内核中进行的一些操作可能导致一些原先存放某个应用代码的物理页帧如今用来存放数据或者是其他应用的代码，
        // i-cache 中可能还保存着该物理页帧的错误快照。
        // 因此我们直接将整个 i-cache 清空避免错误。
        // 接着使用 jr 指令完成了跳转到 __restore 的任务。
        core::arch::asm!(
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,
            in("a1") user_satp,
            options(noreturn)
        );
    }
}
