use crate::trap::trap_return;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TaskContext {
    ra: usize,
    sp: usize,
    s: [usize; 12],
}

impl TaskContext {
    pub fn init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }

    /// os3 用于创建新的任务, 传入任务的入口地址
    #[allow(unused)]
    pub fn goto_restore(kstack_ptr: usize) -> Self {
        extern "C" {
            fn __restore(); // no need pass any argument
        }
        Self {
            ra: __restore as usize, // ra = __restore
            sp: kstack_ptr,
            // 内核态第一次进入用户态执行用户态
            // 此时 __switch 加载的 TaskContext 是由 TaskContext::goto_restore 生成的，
            // 可以看到里面的 s0-s11 均为 0，也就是并不带有任何信息，只是起到一个占位作用,
            // 真正有意义的是 TaskContext中 ra 和 sp 两个寄存器的值，
            // 它们能帮助我们从内核栈的位置开始执行 __restore 回到用户态;
            // 这个过程中 s0-s11 会被覆盖，但正如之前所说这些寄存器的值目前本来就是无意义的，可以随意覆盖
            s: [0; 12],
        }
    }

    // os4
    pub fn goto_trap_return(kstack_ptr: usize) -> Self {
        // 在构造方式上，只是将 ra 寄存器的值设置为 trap_return 的地址。 trap_return 是 os4 新版的 Trap 处理的一部分。
        Self {
            // 当每个应用第一次获得 CPU 使用权即将进入用户态执行的时候，它的内核栈顶放置着我们在 内核加载应用的时候 构造的一个任务上下文
            // 在 __switch 切换到该应用的任务上下文的时候，内核将会跳转到 trap_return 并返回用户态开始该应用的启动执行。
            ra: trap_return as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }
}
