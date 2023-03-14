//! Memory management implementation
//!
//! SV39 page-based virtual-memory architecture for RV64 systems, and
//! everything about memory management, like frame allocator, page table,
//! map area and memory set, is implemented here.
//!
//! Every task or process has a memory_set to control its virtual memory.

mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use address::{StepByOne, VPNRange};
pub use frame_allocator::{frame_alloc, FrameTracker};
pub use memory_set::remap_test;
pub use memory_set::{MapPermission, MemorySet, KERNEL_SPACE};
pub use page_table::{translated_byte_buffer, translated_mut, PageTableEntry};
pub use page_table::{PTEFlags, PageTable};

/// initiate heap allocator, frame allocator and kernel space
pub fn init() {
    // 先进行全局动态内存分配器的初始化，因为接下来马上就要用到 Rust 的堆数据结构
    heap_allocator::init_heap();

    // 初始化物理页帧管理器（内含堆数据结构 Vec<T> ）使能可用物理页帧的分配和回收能力。
    frame_allocator::init_frame_allocator();

    // 创建内核地址空间并让 CPU 开启分页模式
    // 引用 KERNEL_SPACE ，这是它第一次被使用，就在此时它会被初始化，调用 MemorySet::new_kernel 创建一个内核地址空间并使用 Arc<Mutex<T>> 包裹起来
    // lock 返回一个 MutexGuard，它是一个智能指针，生命周期结束后互斥锁就会被释放
    KERNEL_SPACE.lock().activate();
}
