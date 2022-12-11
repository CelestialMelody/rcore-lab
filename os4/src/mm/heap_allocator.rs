use crate::config::KERNEL_HEAP_SIZE;
use buddy_system_allocator::LockedHeap;

#[global_allocator] // 使用 alloc 要求的 #[global_allocator] 语义项进行标记
/// The global allocator.
// 将 buddy_system_allocator 中提供的 LockedHeap 实例化成一个全局变量
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

// LockedHeap 是一个被互斥锁 Mutex<T> 保护的类型，在对它任何进行任何操作之前都要先获取锁以避免其他线程同时对它进行操作导致数据竞争
// LockedHeap 实现了 GlobalAlloc 要求的抽象接口, 用于分配内存

// 这块内存是一个 static mut 且被零初始化的字节数组，位于内核的 .bss 段中
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

/// 在使用任何 alloc 中提供的堆数据结构之前，
/// 需要先调用 init_heap 函数来给全局分配器一块内存用于分配;
pub fn init() {
    unsafe {
        HEAP_ALLOCATOR
            .lock() // 获取锁
            // LockedHeap 是一个被互斥锁 Mutex<T> 保护的类型，在对它任何进行任何操作之前都要先获取锁以避免其他线程同时对它进行操作导致数据竞争
            .init(HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE); // 调用 init 方法告知它能够用来分配的空间的起始地址和大小
    }
}

#[alloc_error_handler]
// alloc_error_handler is a lang item, see https://doc.rust-lang.org/nightly/core/alloc/trait.GlobalAlloc.html#method.alloc
/// 处理动态内存分配失败的情形 -> 直接 panic
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Allocation error, layout = {:?}", layout);
}

#[allow(unused)]
pub fn heap_test() {
    init();
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
    for i in 0..500 {
        assert_eq!(v[i], i);
    }
    assert!(bss_range.contains(&(v.as_ptr() as usize)));
    drop(v);
    println!("heap_test passed!");
}

// to write test program, see https://os.phil-opp.com/zh-CN/testing/
// #[cfg(test)]
// mod tests {
//     use crate::heap_alloc::init;
//     #[test]
//     #[no_std]
//     pub fn heap_test() {
//         init();
//         use alloc::boxed::Box;
//         use alloc::vec::Vec;
//         extern "C" {
//             fn sbss();
//             fn ebss();
//         }
//         let bss_range = sbss as usize..ebss as usize;
//         let a = Box::new(5);
//         assert_eq!(*a, 5);
//         assert!(bss_range.contains(&(a.as_ref() as *const _ as usize)));
//         drop(a);
//         let mut v: Vec<usize> = Vec::new();
//         for i in 0..500 {
//             v.push(i);
//         }
//         for i in 0..500 {
//             assert_eq!(v[i], i);
//         }
//         assert!(bss_range.contains(&(v.as_ptr() as usize)));
//         drop(v);
//         println!("heap_test passed!");
//     }
// }
