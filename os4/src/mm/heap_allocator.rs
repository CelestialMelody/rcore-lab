//! The global allocator

use crate::config::KERNEL_HEAP_SIZE;
use buddy_system_allocator::LockedHeap;

// 使用 alloc 要求的 #[global_allocator] 语义项进行标记
#[global_allocator]

/// heap allocator instance
// 将 buddy_system_allocator 中提供的 LockedHeap 实例化成一个全局变量
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
/// panic when heap allocation error occurs
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

/// heap space ([u8; KERNEL_HEAP_SIZE])
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];
// 这块内存是一个 static mut 且被零初始化的字节数组，位于内核的 .bss 段中

/// initiate heap allocator
pub fn init_heap() {
    // 调用 init_heap 函数来给全局分配器一块内存用于分配
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
}

#[allow(unused)]
pub fn heap_test() {
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    extern "C" {
        fn sbss();
        fn ebss();
    }
    let bss_range = sbss as usize..ebss as usize;
    let a = Box::new(5);
    assert_eq!(*a, 5);
    assert!(bss_range.contains(&(a.as_ref() as *const _ as usize)));
    drop(a);
    let mut v: Vec<usize> = Vec::new();
    for i in 0..500 {
        v.push(i);
    }
    for (i, vi) in v.iter().enumerate().take(500) {
        assert_eq!(*vi, i);
    }
    assert!(bss_range.contains(&(v.as_ptr() as usize)));
    drop(v);
    info!("heap_test passed!");
}

// 补充

// buddy_system_allocator::LockedHeap 是一个被互斥锁 Mutex<T> 保护的类型，在对它任何进行任何操作之前都要先获取锁以避免其他线程同时对它进行操作导致数据竞争
// LockedHeap 实现了 GlobalAlloc 要求的抽象接口, 用于分配内存

// [alloc_error_handler] see https://doc.rust-lang.org/nightly/core/alloc/trait.GlobalAlloc.html#method.alloc
