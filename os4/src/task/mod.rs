//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the operating system.
//!
//! Be careful when you see [`__switch`]. Control flow around this function
//! might not be what you expect.

mod context;
mod switch;

#[allow(clippy::module_inception)] // 允许有与其父模块同名的子模块
mod task;

use crate::config::{MAX_APP_NUM, MAX_SYSCALL_NUM};
use crate::loader::{get_app_data, get_num_apps};
use crate::sync::UnSafeCell;
use crate::timer::get_time_micro;
use crate::trap::TrapContext;
use alloc::vec::Vec;
pub use context::TaskContext;
use lazy_static::*;
pub use switch::__switch;
pub use task::{TaskControlBlock, TaskStatus};

pub struct TaskManager {
    /// number of tasks
    num_apps: usize,
    /// use inner value to get mutbale reference
    pub inner: UnSafeCell<TaskManagerInner>,
}

pub struct TaskManagerInner {
    /// 任务列表
    pub tasks: Vec<TaskControlBlock>,
    /// 用于记录当前正在运行的任务id
    pub current_task: usize,
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        info!("init TASK_MANAGER");

        let num_apps = get_num_apps();

        info!("num_apps: {}", num_apps);

        // let mut tasks = [TaskControlBlock::new(); MAX_APP_NUM]; // out of memory

        // 在 TaskManagerInner 中我们使用向量 Vec 来保存任务控制块。
        // 在全局任务管理器 TASK_MANAGER 初始化的时候，只需使用 loader 子模块提供的 get_num_app 和 get_app_data
        // 分别获取链接到内核的应用数量和每个应用的 ELF 文件格式的数据，然后依次给每个应用创建任务控制块并加入到向量中即可。

        let mut tasks: Vec<TaskControlBlock> = Vec::with_capacity(MAX_APP_NUM);
        for i in 0..num_apps {
            tasks.push(TaskControlBlock::new(get_app_data(i), i));
        }

        TaskManager {
            num_apps,
            inner: unsafe {
                UnSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}

impl TaskManager {
    /// Run the first task in task list.
    ///
    /// Generally, the first task in task list is an idle task (we call it zero process later).
    /// But in ch4, we load apps statically, so the first task is a real app.
    ///
    fn run_first_task(&self) -> ! {
        // get the first task
        let mut inner = self.inner.exclusive_access();

        // 取出即将最先执行的编号为 0 的应用的任务上下文指针 next_task_cx_ptr 并希望能够切换过去
        let task_zero = &mut inner.tasks[0];
        task_zero.task_status = TaskStatus::Running;

        let next_task_cx_ptr = &task_zero.task_cx as *const TaskContext;

        // lab1
        // change task_zero before drop inner
        task_zero.begin_time = get_time_micro();
        task_zero.is_first_run = false;

        drop(inner); // 释放 exclusive_access() 的锁

        // run the first task
        // 在 run_first_task 的时候，我们并没有执行任何应用， __switch 前半部分的保存仅仅是在启动栈上保存了一些之后不会用到的数据，自然也无需记录启动栈栈顶的位置。
        // 因此，我们显式在启动栈上分配了一个名为 `_unused` 的任务上下文，并将它的地址作为第一个参数传给 `__switch` ，这样保存一些寄存器之后的启动栈栈顶的位置将会保存在此变量中。
        // 然而无论是此变量还是启动栈我们之后均不会涉及到，一旦应用开始运行，我们就开始在应用的用户栈和内核栈之间开始切换了。这里声明此变量的意义仅仅是为了避免覆盖到其他数据。
        let mut _unused_cx = TaskContext::init();

        unsafe {
            debug!("run_first_task: __switch");
            __switch(&mut _unused_cx as *mut TaskContext, next_task_cx_ptr);
            debug!("run_first_task: __switch end");
        }
        // 通过 __swtich 设置了 sp, ra 寄存器的值 sp(kernel_stack),ra(tarp_return), switch 结束之后就会跳转到 __restore(trap_return -> 'jr __restore')
        panic!("unreachable in run_first_task");
    }

    /// change the status of current task form Running to Ready
    fn mark_currunt_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let curr_task_id = inner.current_task;
        inner.tasks[curr_task_id].task_status = TaskStatus::Ready;
        // no need drop inner, because it will be dropped when out of scope
    }

    /// change the status of next task form Ready to Exited
    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let curr_task_id = inner.current_task;
        inner.tasks[curr_task_id].task_status = TaskStatus::Exited;

        // lab1
        // acturally, we no need to do this
        inner.tasks[curr_task_id].end_time = get_time_micro();
    }

    /// find the next task to run and return its id
    /// currently, return the first Ready task in the tasks array
    fn find_next_task(&self) -> Option<usize> {
        // error handling: Option<usize> 表示可能找到下一个任务，也可能没有找到;
        let inner = self.inner.exclusive_access();
        let curr_task_id = inner.current_task;

        (curr_task_id + 1..curr_task_id + self.num_apps + 1)
            .map(|id| id % self.num_apps) // 保证 id 一定在 [0, self.num_apps) 范围内
            .find(|&id| inner.tasks[id].task_status == TaskStatus::Ready)
    }

    fn run_next_task(&self) {
        if let Some(next_task_id) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();

            let curr_task_id = inner.current_task;

            let curr_task_cx_ptr = &mut inner.tasks[curr_task_id].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next_task_id].task_cx as *const TaskContext;

            // 由于 mark_currunt_suspended() or mark_current_exited() 已经将当前任务的状态改为 Ready 或 Exited，所以这里不需要再改变状态;
            // see suspend_current_and_run_next() and exit_current_and_run_next()
            // inner.tasks[curr_task_id].task_status = TaskStatus::Ready;
            inner.tasks[next_task_id].task_status = TaskStatus::Running;

            inner.current_task = next_task_id;

            // lab1
            if inner.tasks[next_task_id].is_first_run {
                inner.tasks[next_task_id].begin_time = get_time_micro();
                inner.tasks[next_task_id].is_first_run = false;
            }

            // 因为一般情况下它是在函数退出之后才会被自动释放，从而 TASK_MANAGER 的 inner 字段得以回归到未被借用的状态，之后可以再借用。
            // 如果不手动 drop 的话，编译器会在 __switch 返回时，也就是当前应用被切换回来的时候才 drop，这期间我们都不能修改 TaskManagerInner ，
            // 甚至不能读（因为之前是可变借用），会导致内核 panic 报错退出。
            // 正因如此，我们需要在 __switch 前提早手动 drop 掉 inner
            drop(inner); // drop the local variable inner

            unsafe {
                __switch(curr_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            panic!("no task to run");
        }
    }
}

// lab1
impl TaskManager {
    // acturally, we no need to do this, it must be Running
    fn get_curr_task_status(&self) -> TaskStatus {
        let inner = self.inner.get_ref();
        let curr_task_id = inner.current_task;
        inner.tasks[curr_task_id].task_status
    }

    fn get_curr_task_syscall_times(&self) -> [u32; MAX_SYSCALL_NUM] {
        let inner = self.inner.get_ref();
        let task_id = inner.current_task;
        inner.tasks[task_id].syscall_times
    }

    fn record_curr_task_syscall_times(&self, syscall_id: usize) {
        let mut inner = self.inner.exclusive_access();
        let task_id = inner.current_task;
        inner.tasks[task_id].syscall_times[syscall_id] += 1;
    }

    fn get_curr_task_running_time(&self) -> usize {
        let inner = self.inner.get_ref();
        let task_id = inner.current_task;
        let begin_time = inner.tasks[task_id].begin_time;

        match inner.tasks[task_id].task_status {
            // acturally, we no need to do this, it must be Running
            TaskStatus::Exited => {
                let end_time = inner.tasks[task_id].end_time;
                end_time - begin_time
            }
            _ => get_time_micro() - begin_time,
        }
    }

    fn get_task_status(&self, task_id: usize) -> TaskStatus {
        let inner = self.inner.get_ref();
        inner.tasks[task_id].task_status
    }
}

// lab2
impl TaskManager {
    /// Get the current 'Running' task's token.
    fn get_current_token(&self) -> usize {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].user_token()
    }

    #[allow(clippy::mut_from_ref)]
    /// Get the current 'Running' task's trap contexts.
    fn get_current_trap_cx(&self) -> &mut TrapContext {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].trap_cx()
    }

    fn mmap(&self, va: usize, size: usize, mark: usize) -> isize {
        let mut inner = self.inner.exclusive_access();
        let task_id = inner.current_task;
        inner.tasks[task_id].mmap(va, size, mark)
    }

    fn munmap(&self, va: usize, size: usize) -> isize {
        let mut inner = self.inner.exclusive_access();
        let task_id = inner.current_task;
        inner.tasks[task_id].munmap(va, size)
    }
}

/// call from rust_main,
/// before call this function,
/// use lazy_static to init TASK_MANAGER
pub fn run_first_task() {
    TASK_MANAGER.run_first_task()
}

fn mark_currunt_suspended() {
    TASK_MANAGER.mark_currunt_suspended();
}

fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

pub fn suspend_current_and_run_next() {
    mark_currunt_suspended();
    run_next_task();
}

pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

pub fn get_task_status(task_id: usize) -> TaskStatus {
    TASK_MANAGER.get_task_status(task_id)
}

// lab1
pub fn get_curr_task_status() -> TaskStatus {
    TASK_MANAGER.get_curr_task_status()
}

pub fn get_curr_task_syscall_times() -> [u32; MAX_SYSCALL_NUM] {
    TASK_MANAGER.get_curr_task_syscall_times()
}

pub fn record_curr_task_syscall_times(syscall_id: usize) {
    TASK_MANAGER.record_curr_task_syscall_times(syscall_id)
}

pub fn get_curr_task_running_time() -> usize {
    TASK_MANAGER.get_curr_task_running_time()
}

// lab2
// 通过 current_user_token 和 current_trap_cx 分别可以获得当前正在执行的应用的地址空间的 token 和可以在内核地址空间中修改位于该应用地址空间中的 Trap 上下文的可变引用。

/// Get the current 'Running' task's token.
pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}

/// Get the current 'Running' task's trap contexts.
pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}

pub fn mmap(va: usize, size: usize, mark: usize) -> isize {
    TASK_MANAGER.mmap(va, size, mark)
}

pub fn munmap(va: usize, size: usize) -> isize {
    TASK_MANAGER.munmap(va, size)
}
