core::arch::global_asm!(include_str!("switch.S"));

use super::TaskContext;

extern "C" {
    // fn __switch(from: *mut TaskContext, to: *const TaskContext);
    /// current task context is mut
    /// next task context is const
    pub fn __switch(curr_task_cx_ptr: *mut TaskContext, next_task_cx_ptr: *const TaskContext);
}
