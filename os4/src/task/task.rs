use super::TaskContext;
use super::TaskContext;
use crate::config::{kernel_stack_position, MAX_SYSCALL_NUM, TRAP_CONTEXT};
use crate::mm::{MapPermission, MemorySet, PhysPageNum, VirtAddr, KERNEL_SPACE};
use crate::trap::{trap_handler, TrapContext};

#[derive(Clone, Copy)]
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,

    // lab1
    pub syscall_times: [u32; MAX_SYSCALL_NUM], // solve way: use vec
    pub first_run: bool,
    pub begin_time: usize,
    pub end_time: usize,

    // 应用的地址空间
    pub memory_set: MemorySet,
    // 应用地址空间次高页的 Trap 上下文被实际存放在物理页帧的物理页号 trap_cx_ppn ，它能够方便我们对于 Trap 上下文进行访问
    pub trap_cx_ppn: PhysPageNum,
    // base_size 统计了应用数据的大小，也就是在应用地址空间中从 0x0 开始到用户栈结束一共包含多少字节(用户栈顶位置)。
    // 它后续还应该包含用于应用动态内存分配的堆空间的大小，但目前暂不支持
    pub base_size: usize,
}

impl TaskControlBlock {
    pub fn trap_cx(&self) -> &'static mut TrapContext {
        // 返回 'static 的可变引用和之前一样可以看成一个绕过 unsafe 的裸指针；
        // 而 PhysPageNum::get_mut 是一个泛型函数，由于我们已经声明了总体返回 TrapContext 的可变引用，
        // 则Rust编译器会给 get_mut 泛型函数针对具体类型 TrapContext 的情况生成一个特定版本的 get_mut 函数实现。
        // 在 trap_cx 函数中则会静态调用 get_mut 泛型函数的特定版本实现。
        self.trap_cx_ppn.get_mut()
    }

    pub fn user_token(&self) -> usize {
        self.memory_set.token()
    }

    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        // 解析传入的 ELF 格式数据构造应用的地址空间 memory_set 并获得其他信息
        // memory_set with elf program headers/trampoline/trap context/ user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);

        // 从地址空间 memory_set 中查多级页表找到应用地址空间中的 Trap 上下文实际被放在哪个物理页帧
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();

        let task_status = TaskStatus::Ready;

        // map a kernel stack in kernel space
        // 根据传入的应用 ID app_id 调用在 config 子模块中定义的 kernel_stack_position 找到 应用的内核栈预计放在内核地址空间 KERNEL_SPACE 中的哪个位置，
        // 并通过 insert_framed_area 实际将这个逻辑段 加入到内核地址空间中
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(app_id);
        KERNEL_SPACE.lock().insert_framed_area(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            MapPermission::R | MapPermission::W,
        );

        let task_control_block = Self {
            task_status,
            // 在应用的内核栈顶压入一个跳转到 trap_return 而不是 __restore 的任务上下文，这主要是为了能够支持对该应用的启动并顺利切换到用户地址空间执行。
            task_cx: TaskContext::goto_trap_return(kernel_stack_top),

            syscall_times: [0; MAX_SYSCALL_NUM],
            first_run: true,
            begin_time: 0,
            end_time: 0,

            memory_set,
            trap_cx_ppn,
            base_size: user_sp,
        };

        // prepare TrapContext in user space
        // 查找该应用的 Trap 上下文的内核虚地址。
        // 由于应用的 Trap 上下文是在应用地址空间而不是在内核地址空间中，我们只能手动查页表找到 Trap 上下文实际被放在的物理页帧，
        // 然后通过之前介绍的 在内核地址空间读写特定物理页帧的能力(http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter4/4sv39-implementation-2.html#access-frame-in-kernel-as)
        // 获得在用户空间的 Trap 上下文的可变引用用于初始化
        let trap_cx = task_control_block.trap_cx();

        // 通过应用的 Trap 上下文的可变引用来对其进行初始化
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.lock().token(),
            kernel_stack_top,
            tarp_handler as usize,
        );

        task_control_block
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}
