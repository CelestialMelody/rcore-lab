use super::address::PhysPageNum;

use crate::{config::MEMORY_END, mm::address::PhysAddr, sync::UnSafeCell};

use alloc::vec::Vec;
use core::fmt::{self, Debug, Formatter};
use lazy_static::*;

pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    // 这个物理页帧之前可能被分配过并用做其他用途，在这里直接将这个物理页帧上的所有字节清零
    pub fn new(ppn: PhysPageNum) -> Self {
        let byte_array = ppn.get_byte_array();
        for i in byte_array {
            *i = 0;
        }
        Self { ppn }
    }
}

impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FramTracker: PPN={:#x}", self.ppn.0))
    }
}

// 为 FrameTracker 实现 Drop Trait
impl Drop for FrameTracker {
    // 当一个 FrameTracker 生命周期结束被编译器回收的时候，需要将它控制的物理页帧回收到 FRAME_ALLOCATOR 中
    // 当一个 FrameTracker 实例被回收的时候，它的 drop 方法会自动被编译器调用，
    // 通过之前实现的 frame_dealloc 将它控制的物理页帧回收 以供后续使用
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

/// 物理页帧管理器;
/// 物理页号为单位进行物理页帧的分配和回收
trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}

/// 物理页号区间 [ current , end ) 此前均从未被分配出去过，
/// 而向量 recycled 以后入先出的方式保存了被回收的物理页号
pub struct StackFrameAllocator {
    current: usize, // 空闲内存的起始物理页号
    end: usize,     // 空闲内存的结束物理页号
    recycled: Vec<usize>,
}

impl StackFrameAllocator {
    /// 它真正被使用起来之前，需要调用 init 方法将自身的 [current, end) 初始化为可用物理页号区间
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
        // 首先会检查栈 recycled 内有没有之前回收的物理页号，如果有的话直接弹出栈顶并返回
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else if self.current == self.end {
            // 极端情况下可能出现内存耗尽分配失败的情况：
            // 即 recycled 为空且 current == end;
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
    /// 检查回收页面的合法性，然后将其压入 recycled 栈中
    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        // https://rustwiki.org/zh-CN/core/iter/trait.Iterator.html#method.any
        // 回收页面合法有两个条件：
        // 1. 该页面之前一定被分配出去过，因此它的物理页号一定  < current ；
        // 2. 该页面没有被回收过，即它的物理页号不能在栈 recycled 中找到
        if ppn >= self.current || self.recycled.iter().any(|v| *v == ppn) {
            panic!("Frame ppn={:?} has not been allocated", ppn);
        }
        self.recycled.push(ppn);
    }
}

/// 定义类型别名
/// type 不是创建一个新的类型
/// type 是为已有的类型创建一个新的名字
type FrameAllocatorImpl = StackFrameAllocator;

lazy_static! {
    pub static ref FRAME_ALLOCATOR: UnSafeCell<FrameAllocatorImpl> =
        unsafe { UnSafeCell::new(FrameAllocatorImpl::new()) };
}

/// 物理页帧全局管理器 FRAME_ALLOCATOR 初始化
pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR.exclusive_access().init(
        // 调用物理地址 PhysAddr 的 floor/ceil 方法分别下/上取整获得可用的物理页号区间
        PhysAddr::from(ekernel as usize).ceil(), // ceil 向上取整
        PhysAddr::from(MEMORY_END).floor(),      // floor 向下取整
    );
}

// 公开给其他内核模块调用的分配物理页帧的接口
// frame_alloc 的返回值类型并不是 FrameAllocator 要求的物理页号 PhysPageNum ，而是将其进一步包装为一个 FrameTracker 。
// 这里借用了 RAII 的思想，将一个物理页帧的生命周期绑定到一个 FrameTracker 变量上，
// 当一个 FrameTracker 被创建的时候，我们需要从 FRAME_ALLOCATOR 中分配一个物理页帧;
// 从其他内核模块的视角看来，物理页帧分配的接口是调用 frame_alloc 函数得到一个 FrameTracker （如果物理内存还有剩余），
// 它就代表了一个物理页帧，当它的生命周期结束之后它所控制的物理页帧将被自动回收
pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc()
        // https://rustwiki.org/zh-CN/core/option/enum.Option.html#method.map
        // 将分配来的物理页帧的物理页号作为参数传给 FrameTracker 的 new 方法来创建一个 FrameTracker 实例
        .map(FrameTracker::new)
}

// 公开给其他内核模块调用的回收物理页帧的接口
fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
}

#[allow(unused)]
pub fn frame_allocator_test() {
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        info!("frame {} = {:?}", i, frame);
        v.push(frame); // without this frame will be dropped when it goes out of scope
                       // 将 frame move 到一个向量中，生命周期被延长了
    }
    info!("before clear: vec = {:?}, len = {}", v, v.len());
    v.clear();
    info!("after clear: vec = {:?}, len = {}", v, v.len());
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        info!("frame {} = {:?}", i, frame);
        v.push(frame); // frame 会调用 drop
    }
    drop(v);
    info!("frame_allocator_test pass");
}
