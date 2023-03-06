//! Implementation of [`FrameAllocator`] which
//! controls all the frames in the operating system.

use super::{PhysAddr, PhysPageNum};
use crate::config::MEMORY_END;
use crate::sync::UnSafeCell;
use alloc::vec::Vec;
use core::fmt::{self, Debug, Formatter};
use lazy_static::*;

/// manage a frame which has the same lifecycle as the tracker
pub struct FrameTracker {
    pub ppn: PhysPageNum,
}
impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        // 由于这个物理页帧之前可能被分配过并用做其他用途，直接将这个物理页帧上的所有字节清零
        let bytes_array = ppn.get_bytes_array(); // 4K
        for i in bytes_array {
            *i = 0;
        }
        Self { ppn }
    }
}
impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN={:#x}", self.ppn.0))
    }
}
impl Drop for FrameTracker {
    // 当一个 FrameTracker 实例被回收的时候，它的 drop 方法会自动被编译器调用
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

/// 物理页帧管理器，以物理页号为单位进行物理页帧的分配和回收
trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}

/// an implementation for frame allocator
pub struct StackFrameAllocator {
    current: usize, // 空闲内存的起始物理页号
    end: usize,     // 空闲内存的结束物理页号
    // 物理页号区间 [ current , end ) 此前均从未被分配出去过，
    recycled: Vec<usize>,
    // 而向量 recycled 以后入先出的方式保存了被回收的物理页号
}
impl StackFrameAllocator {
    /// 将自身的 [current, end) 初始化为可用物理页号区间
    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l.0;
        self.end = r.0;
    }
}
impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }
    /// 物理页帧分配
    fn alloc(&mut self) -> Option<PhysPageNum> {
        // 首先检查栈 recycled 内有没有之前回收的物理页号，如果有的话直接弹出栈顶并返回
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else if self.current == self.end {
            // 极端情况下可能出现内存耗尽分配失败的情况，即 recycled 为空且 current == end;
            // 为了涵盖这种情况， alloc 的返回值被 Option 包裹，返回 None 即可
            None
        } else {
            // 从之前从未分配过的物理页号区间 [ current , end ) 上进行分配，
            // 分配它的左端点 current ，同时将管理器内部维护的 current 加 1 代表 current 已被分配了
            self.current += 1;
            // 使用 into 方法将 usize 转换成了物理页号 PhysPageNum
            Some((self.current - 1).into())
        }
    }
    /// 物理页帧回收
    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        // 检查回收页面的合法性，然后将其压入 recycled 栈中
        // 回收页面合法有两个条件：
        // 1. 该页面之前一定被分配出去过，因此它的物理页号一定  < current ；
        // 2. 该页面没有被回收过，即它的物理页号不能在栈 recycled 中找到
        if ppn >= self.current || self.recycled.iter().any(|v| *v == ppn) {
            // 如果回收页面不合法，即 ppn >= current 或者 ppn 在 recycled 中找到了，就 panic
            panic!("Frame ppn={:?} has not been allocated", ppn);
        }
        self.recycled.push(ppn);
    }
}

// 类型别名
type FrameAllocatorImpl = StackFrameAllocator;

lazy_static! {
    /// frame allocator instance through lazy_static!
    pub static ref FRAME_ALLOCATOR: UnSafeCell<FrameAllocatorImpl> =
        unsafe { UnSafeCell::new(FrameAllocatorImpl::new()) };
}

/// initiate the frame allocator using `ekernel` and `MEMORY_END`
pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR.exclusive_access().init(
        // 调用物理地址 PhysAddr 的 floor/ceil 方法分别下/上取整获得可用的物理页号区间
        PhysAddr::from(ekernel as usize).ceil(),
        PhysAddr::from(MEMORY_END).floor(),
    );
}

/// allocate a frame
pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc()
        // 将分配来的物理页帧的物理页号作为参数传给 FrameTracker 的 new 方法来创建一个 FrameTracker 实例
        .map(FrameTracker::new)
}

/// deallocate a frame
fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
}

#[allow(unused)]
/// a simple test for frame allocator
pub fn frame_allocator_test() {
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        info!("{:?}", frame);
        v.push(frame); // 将 frame move 到一个向量中，生命周期被延长了
    }
    v.clear();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        info!("{:?}", frame);
        v.push(frame);
    }
    drop(v);
    info!("frame_allocator_test passed!");
}

// 补充

// [amy](https://rustwiki.org/zh-CN/core/iter/trait.Iterator.html#method.any)
// [map](https://rustwiki.org/zh-CN/core/option/enum.Option.html#method.map)

// fn frame_alloc 的返回值类型并不是 FrameAllocator 要求的物理页号 PhysPageNum ，
// 而是将其进一步包装为一个 FrameTracker。借用了 RAII 的思想，将一个物理页帧的生命周期绑定到一个 FrameTracker 变量上，
// 当一个 FrameTracker 被创建的时候，我们需要从 FRAME_ALLOCATOR 中分配一个物理页帧;
// 从其他内核模块的视角看来，物理页帧分配的接口是调用 frame_alloc 函数得到一个 FrameTracker （如果物理内存还有剩余），
// 它就代表了一个物理页帧，当它的生命周期结束之后它所控制的物理页帧将被自动回收。
