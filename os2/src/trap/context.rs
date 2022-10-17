use riscv::register::sstatus::{self, Sstatus, SPP};

#[repr(C)] //数据对齐属性; repr(C) 表示按照 C 语言的方式对齐
/// Trap 上下文，在 Trap 发生时需要保存的物理资源内容
/** - 对于通用寄存器而言，两条控制流（应用程序控制流和内核控制流）运行在不同的特权级，
      所属的软件也可能由不同的编程语言编写，虽然在 Trap 控制流中只是会执行 Trap 处理 相关的代码，
      但依然可能直接或间接调用很多模块，因此很难甚至不可能找出哪些寄存器无需保存。
      既然如此我们就只能全部保存了。但这里也有一些例外， 如 x0 被硬编码为 0 ，它自然不会有变化；
      还有 tp(x4) 寄存器，除非我们手动出于一些特殊用途使用它，否则一般也不会被用到。
      虽然它们无需保存， 但我们仍然在 TrapContext 中为它们预留空间，主要是为了后续的实现方便。
*/
/** - 对于 CSR 而言，我们知道进入 Trap 的时候，硬件会立即覆盖掉 scause/stval/sstatus/sepc 的全部或是其中一部分。
      scause/stval 的情况是：它总是在 Trap 处理的第一时间就被使用或者是在其他地方保存下来了，因此它没有被修改并造成不良影响的风险。 
      而对于 sstatus/sepc 而言，它们会在 Trap 处理的全程有意义（在 Trap 控制流最后 sret 的时候还用到了它们），
      而且确实会出现 Trap 嵌套的情况使得它们的值被覆盖掉。所以我们需要将它们也一起保存下来，并在 sret 之前恢复原样。
*/
// 1. size = 34 * 8 Bytes -> see trap.S
// 2. 不要改成员顺序; 内存布局 -> trap.S
pub struct TrapContext { 
    // 保存寄存器的值; 通用寄存器 x0~x31
    pub x: [usize; 32],
    // 保存异常发生时的程序状态寄存器
    pub sstatus: Sstatus,
    // 保存异常发生时的程序计数器
    pub sepc: usize,
}


/// 当批处理操作系统初始化完成，或者是某个应用程序运行结束或出错的时候，调用 run_next_app 函数切换到下一个应用程序
/// 此时 CPU 运行在 S 特权级，而它希望能够切换到 U 特权级。在 RISC-V 架构中，唯一一种能够使得 CPU 特权级下降的方法就是执行 Trap 返回的特权指令，如 sret 、mret 等。
/// 事实上，在从操作系统内核返回到运行应用程序之前，要完成如下这些工作：
/// - 构造应用程序开始执行所需的 Trap 上下文；
/// - 通过 __restore 函数，从刚构造的 Trap 上下文中，恢复应用程序执行的部分寄存器；
/// - 设置 sepc CSR的内容为应用程序入口点 0x80400000；
/// - 切换 scratch 和 sp 寄存器，设置 sp 指向应用程序用户栈；
/// - 执行 sret 从 S 特权级切换到 U 特权级。
/// 它们可以通过复用 __restore 的代码来更容易的实现上述工作。
/// 我们只需要在内核栈上压入一个为启动应用程序而特殊构造的 Trap 上下文，再通过 __restore 函数，就能让这些寄存器到达启动应用程序所需要的上下文状态。
impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp; // x2 is sp
    }
    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        let mut sstatus = sstatus::read(); // 读取 sstatus 寄存器
        sstatus.set_spp(SPP::User);
        // 修改 sepc 寄存器为应用程序入口点 entry(APP_BASE_ADDRESS)， 
        // sp 寄存器为我们设定的一个栈指针，
        // 并将 sstatus 寄存器的 SPP 字段设置为 User
        let mut context = Self {
            x: [0; 32],
            sepc: entry, // APP_BASE_ADDRESS: 0x80400000
            sstatus,
        };
        context.set_sp(sp);
        context
    }
}