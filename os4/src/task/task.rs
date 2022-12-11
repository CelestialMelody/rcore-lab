use crate::config::MAX_SYSCALL_NUM;

use super::TaskContext;

#[derive(Clone, Copy)]
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,

    // lab1
    pub syscall_times: [u32; MAX_SYSCALL_NUM], // solve way: use vec
    pub first_run: bool,
    pub begin_time: usize,
    pub end_time: usize,
}

impl TaskControlBlock {
    pub fn new() -> Self {
        Self {
            task_cx: TaskContext::init(),
            task_status: TaskStatus::UnInit,

            syscall_times: [0; MAX_SYSCALL_NUM],
            first_run: true,
            begin_time: 0,
            end_time: 0,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}
