use super::timer::TimeVal;
use crate::config::MAX_SYSCALL_NUM;
use crate::mm::translated_mut;
use crate::task::{
    current_user_token, exit_current_and_run_next, get_curr_task_running_time,
    get_curr_task_status, get_curr_task_syscall_times, mmap, munmap, suspend_current_and_run_next,
    TaskStatus,
};
use crate::timer::get_time_val;

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
pub fn sys_get_time(mut ts: *mut TimeVal, _tz: usize) -> isize {
    // lab2
    let token = current_user_token();
    ts = translated_mut(token, ts);
    unsafe { *ts = get_time_val() }
    0
}

pub fn sys_set_priority(_prio: isize) -> isize {
    -1
}

// YOUR JOB: 扩展内核以实现 sys_mmap 和 sys_munmap
// tcb -> memory_set -> insert_framed_area
// syscall ID：222
// 申请长度为 len 字节的物理内存（不要求实际物理内存位置，可以随便找一块），将其映射到 start 开始的虚存，内存页属性为 port
// 参数：
// start 需要映射的虚存起始地址，要求按页对齐
// len 映射字节长度，可以为 0
// port：第 0 位表示是否可读，第 1 位表示是否可写，第 2 位表示是否可执行。其他位无效且必须为 0
// 返回值：执行成功则返回 0，错误返回 -1
// 说明：
// 为了简单，目标虚存区间要求按页对齐，len 可直接按页向上取整，不考虑分配失败时的页回收。
// 可能的错误：
// start 没有按页大小对齐
// port & !0x7 != 0 (port 其余位必须为0)
// port & 0x7 = 0 (这样的内存无意义)
// [start, start + len) 中存在已经被映射的页 (如何判断？)
// 物理内存不足 （如何判断？）
// 一定要注意 mmap 是的页表项，注意 riscv 页表项的格式与 port 的区别。
// 增加 PTE_U
pub fn sys_mmap(start_va: usize, len: usize, mark: usize) -> isize {
    mmap(start_va, len, mark);
    0
}

// syscall ID：215
// 取消到 [start, start + len) 虚存的映射
// 参数和返回值请参考 mmap
// 说明：
// 为了简单，参数错误时不考虑内存的恢复和回收。
// 可能的错误：
// [start, start + len) 中存在未被映射的虚存。
pub fn sys_munmap(start: usize, len: usize) -> isize {
    munmap(start, len);
    0
}

/// 获取当前任务的信息
// YOUR JOB: 引入虚地址后重写 sys_task_info
pub fn sys_task_info(mut ti: *mut TaskInfo) -> isize {
    let token = current_user_token();
    ti = translated_mut(token, ti);
    unsafe {
        (*ti).status = get_curr_task_status();
        (*ti).syscall_times = get_curr_task_syscall_times();
        (*ti).time = get_curr_task_running_time() / 1000;
    }
    0
}
