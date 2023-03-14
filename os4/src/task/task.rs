use super::TaskContext;
use crate::config::{kernel_stack_position, MAX_SYSCALL_NUM, TRAP_CONTEXT};
use crate::mm::{MapPermission, MemorySet, PageTable, PhysPageNum, VirtAddr, KERNEL_SPACE};
use crate::trap::{trap_handler, TrapContext};

pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,

    // lab1
    pub syscall_times: [u32; MAX_SYSCALL_NUM], // solve way: use vec
    pub is_first_run: bool,
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
            // 接着调用 `TaskContext::goto_trap_resturn` 来构造每个任务保存在任务控制块中的任务上下文：
            // 它设置任务上下文中的内核栈指针将任务上下文的 `ra` 设置为 `trap_return` 的入口地址,
            // 这样，在 `__switch` 从它上面恢复并返回之后就会直接跳转到 `trap_return` ，设置 stvec 为 trap_handler 的入口地址，
            // 并前往 __restore 更新 sscratch 的值为 TrapContext (位于应用地址空间)
            // 此时栈顶是一个构造出来第一次进入用户态执行的 Trap 上下文 (返回用户态)
            task_cx: TaskContext::goto_trap_return(kernel_stack_top),

            syscall_times: [0; MAX_SYSCALL_NUM],
            is_first_run: true,
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
            trap_handler as usize,
        );

        task_control_block
    }
}

// lab2
impl TaskControlBlock {
    pub fn mmap(&mut self, va: usize, size: usize, mark: usize) -> isize {
        let va_ = VirtAddr::from(va);
        if !va_.is_aligned() {
            // panic!("va is not aligned");
            return -1;
        }

        if (mark & !0x7) != 0 || (mark & 0x7) == 0 {
            // panic!("mark is not valid");
            return -1;
        }

        // let token = self.user_token();
        // let page_table = PageTable::from_token(token);
        // let vpn = va_.floor();
        // debug!("vpn: {:?}", vpn);
        // debug!("test pte is_some");
        // if page_table.find_pte(vpn).is_some() {
        //     // 似乎没法判断是否有分配，怀疑临时page_table 释放掉frame(中间用于查找页表项的frame)
        //     // panic!("va is already mapped");
        //     debug!("va is already mapped");
        //     debug!("pte is {:?}", page_table.find_pte(vpn).unwrap());
        //     return -1;
        // }
        // debug!("test pte is_some end");

        let vpn = va_.floor();
        debug!("vpn: {:?}", vpn);
        if self.memory_set.is_vpn_mapped(vpn) {
            // panic!("va is already mapped");
            debug!("va is already mapped");
            return -1;
        }

        // mark 与内核定义的 MapPermission 不同
        let mark = mark << 1;
        let mark_ = MapPermission::from_bits_truncate(mark as u8) | MapPermission::U;

        let start_va = va_;
        let end_va = VirtAddr::from(va + size);

        self.memory_set.insert_framed_area(start_va, end_va, mark_);

        0
    }

    pub fn munmap(&mut self, va: usize, size: usize) -> isize {
        let va_ = VirtAddr::from(va);
        if !va_.is_aligned() {
            // panic!("va is not aligned");
            return -1;
        }

        let start_va = va_;
        let end_va = VirtAddr::from(va + size);

        self.memory_set.remove_framed_area(start_va, end_va);

        0
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    // UnInit, // unsued
    Ready,
    Running,
    Exited,
}

// void func(int* i) {
//     i = func_return_int_ptr();
//     *i = 1; // 不会改变a的值
// }

// void func_call() {
//     int a = 0;
//     func(&a);
// }
