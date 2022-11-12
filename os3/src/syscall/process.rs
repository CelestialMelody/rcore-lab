use crate::config::MAX_SYSCALL_NUM;
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next, TaskStatus};
use crate::timer::get_time_micro;

#[repr(C)]
#[derive(Debug)]
/// TimeVal 用于保存时间戳
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

pub struct TaskInfo {
    status: TaskStatus,
    syscall_times: [u32; MAX_SYSCALL_NUM],
    time: usize,
}

/// 打印退出的应用程序的返回值并同样调用 run_next_app 切换到下一个应用程序。
pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// 用于切换到下一个任务
/// current task give up cpu or resouce
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

/// 获取当前时间戳
/// ts 为 TimeVal 类型的指针，用于保存时间戳
/// _tz 为时区，目前不使用
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    let time = get_time_micro();

    unsafe {
        *ts = TimeVal {
            sec: time / 1_000_000, // us -> second
            usec: time % 1_000_000,
        };
    }
    0
}

/// 获取当前任务的信息
pub fn sys_task_info(_task_info: *mut TaskInfo) -> isize {
    // TODO
    -1
}
