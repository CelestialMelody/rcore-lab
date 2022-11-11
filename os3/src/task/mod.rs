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
    /// 类似构造 Trap 上下文的方法，内核需要在应用的任务控制块上构造一个用于第一次执行的任务上下文；
    /// 对于每个任务，我们先调用 `init_app_cx` 构造该任务的 Trap 上下文（包括应用入口地址和用户栈指针）并将其压入到内核栈顶；
    /// 接着调用 `TaskContext::goto_restore` 来构造每个任务保存在任务控制块中的任务上下文：它设置任务上下文中的内核栈指针将任务上下文的 `ra` 寄存器设置为 `__restore` 的入口地址。这样，在 `__switch` 从它上面恢复并返回之后就会直接跳转到 `__restore` ，此时栈顶是一个我们构造出来第一次进入用户态执行的 Trap 上下文，就和第二章的情况一样了。
    fn run_first_task(&self) -> ! {
        // get the first task
        let mut inner = self.inner.exclusive_access();

        // 取出即将最先执行的编号为 0 的应用的任务上下文指针 next_task_cx_ptr 并希望能够切换过去
        let task_zero = &mut inner.tasks[0];
        task_zero.task_status = TaskStatus::Running;

        let next_task_cx_ptr = &task_zero.task_cx as *const TaskContext;

        drop(inner); // 释放 exclusive_access() 的锁

        // run the first task
        // 在 run_first_task 的时候，我们并没有执行任何应用， __switch 前半部分的保存仅仅是在启动栈上保存了一些之后不会用到的数据，自然也无需记录启动栈栈顶的位置。
        // 因此，我们显式在启动栈上分配了一个名为 `_unused` 的任务上下文，并将它的地址作为第一个参数传给 `__switch` ，这样保存一些寄存器之后的启动栈栈顶的位置将会保存在此变量中。
        // 然而无论是此变量还是启动栈我们之后均不会涉及到，一旦应用开始运行，我们就开始在应用的用户栈和内核栈之间开始切换了。这里声明此变量的意义仅仅是为了避免覆盖到其他数据。
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
