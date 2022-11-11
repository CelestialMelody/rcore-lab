mod context;
mod switch;

#[allow(clippy::module_inception)] // 允许有与其父模块同名的子模块
mod task;

use crate::config::MAX_APP_NUM;
use crate::loader::{get_num_apps, init_app_cx};
use crate::sync::UnSafeCell;

use lazy_static::*;

pub use context::TaskContext;
pub use switch::__switch;
pub use task::{TaskControlBlock, TaskStatus};

pub struct TaskManager {
    /// number of tasks
    num_apps: usize,
    /// use inner value to get mutbale reference
    inner: UnSafeCell<TaskManagerInner>,
}

struct TaskManagerInner {
    /// 任务控制块数组
    tasks: [TaskControlBlock; MAX_APP_NUM],
    /// 用于记录当前正在运行的任务id
    current_task: usize,
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_apps = get_num_apps();
        let mut tasks = [TaskControlBlock{
            task_cx: TaskContext::init(),
            task_status: TaskStatus::UnInit,
        }; MAX_APP_NUM];

        for (i, t) in tasks.iter_mut().enumerate().take(num_apps) { // take(num_apps) 保证只初始化 num_apps 个任务; iterater::take() 用于限制迭代器的长度
            t.task_cx = TaskContext::goto_restore(init_app_cx(i)); // 初始化任务上下文
            t.task_status = TaskStatus::Ready;
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
    fn run_first_task(&self) -> ! {
        // get the first task
        let mut inner = self.inner.exclusive_access();
        let task_zero = &mut inner.tasks[0];
        task_zero.task_status = TaskStatus::Running;

        let next_task_cx_ptr = &task_zero.task_cx as *const TaskContext;

        drop(inner); // 释放 exclusive_access() 的锁

        // run the first task
        let mut _unused_cx = TaskContext::init();
        unsafe {
            __switch(&mut _unused_cx as *mut TaskContext, next_task_cx_ptr);
        }

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
    }

    /// find the next task to run and return its id
    /// currently, return the first Ready task in the tasks array
    fn find_next_task(&self) -> Option<usize> {
        // error handling: Option<usize> 表示可能找到下一个任务，也可能没有找到;
        let inner = self.inner.exclusive_access();
        let curr_task_id = inner.current_task;

        //fix: BUG (for sys_yield)
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

            /// 因为一般情况下它是在函数退出之后才会被自动释放，从而 TASK_MANAGER 的 inner 字段得以回归到未被借用的状态，之后可以再借用。
            /// 如果不手动 drop 的话，编译器会在 __switch 返回时，也就是当前应用被切换回来的时候才 drop，这期间我们都不能修改 TaskManagerInner ，
            /// 甚至不能读（因为之前是可变借用），会导致内核 panic 报错退出。
            /// 正因如此，我们需要在 __switch 前提早手动 drop 掉 inner
            drop(inner); // drop the local variable inner

            unsafe {
                __switch(curr_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            panic!("no task to run");
        }
    }
}

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
