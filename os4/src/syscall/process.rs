use crate::config::MAX_SYSCALL_NUM;
use crate::task::{
    exit_current_and_run_next, get_curr_task_running_time, get_curr_task_status,
    get_curr_task_syscall_times, suspend_current_and_run_next, TaskStatus,
};
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
    time: usize, // milliseconds
}

/// 打印退出的应用程序的返回值并同样调用 run_next_app 切换到下一个应用程序。
pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task give up cpu or resouce
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

/// 获取当前时间戳
/// ts 为 TimeVal 类型的指针，用于保存时间戳；
/// _tz 为时区
// YOUR JOB: 引入虚地址后重写 sys_get_time
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    let _us = get_time_micro();
    // unsafe {
    //     *ts = TimeVal {
    //         sec: us / 1_000_000,
    //         usec: us % 1_000_000,
    //     };
    // }
    0
}

pub fn sys_set_priority(_prio: isize) -> isize {
    -1
}

// YOUR JOB: 扩展内核以实现 sys_mmap 和 sys_munmap
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    -1
}

pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    -1
}

/// 获取当前任务的信息
// YOUR JOB: 引入虚地址后重写 sys_task_info
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    -1
    // unsafe {
    //     (*task_info).status = get_curr_task_status();
    //     (*task_info).syscall_times = get_curr_task_syscall_times();
    //     (*task_info).time = get_curr_task_running_time() / 1000;
    // }
    // 0
}
