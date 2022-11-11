core::arch::global_asm!(include_str!("switch.S"));

use super::TaskContext;

extern "C" {
    // fn __switch(from: *mut TaskContext, to: *const TaskContext);
    /// current task context is mut
    /// next task context is const
    /// 当被任务切换出去的应用即将再次运行的时候，它实际上是通过 __switch 函数又完成一次任务切换，只是这次是被切换进来，取得了 CPU 的使用权。
    /// 如果该应用是之前被切换出去的，那么它需要有任务上下文和内核栈上的 Trap 上下文，让切换机制可以正常工作
    /// 注意 __switch 两个参数分别表示当前应用和即将切换到的应用的任务上下文指针，
    /// 其第一个参数存在的意义是记录当前应用的任务上下文被保存在哪里，也就是当前应用内核栈的栈顶，这样之后才能继续执行该应用。
    pub fn __switch(curr_task_cx_ptr: *mut TaskContext, next_task_cx_ptr: *const TaskContext);
}
