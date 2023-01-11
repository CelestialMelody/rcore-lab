use alloc::vec;
use alloc::vec::Vec;
use bitflags::*;

use super::{
    address::{PhysPageNum, StepByOne, VirtAddr, VirtPageNum},
    frame_allocator::{frame_alloc, FrameTracker},
};

bitflags! {
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
/// Page table entry
/// 让编译器自动为 PageTableEntry 实现 Copy/Clone Trait，来让这个类型以值语义赋值/传参的时候不会发生所有权转移，而是拷贝一份新的副本。
/// 从这一点来说 PageTableEntry 就和 usize 一样，因为它也只是后者的一层简单包装，并解释了 usize 各个比特段的含义
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    /// 从一个物理页号 PhysPageNum 和一个页表项标志位 PTEFlags 生成一个页表项 PageTableEntry 实例
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }

    /// empty 方法生成一个全零的页表项，注意这隐含着该页表项的 V 标志位为 0 ，因此它是不合法的
    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }

    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & (1usize << 44) - 1).into()
    }

    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }

    pub fn is_valid(&self) -> bool {
        self.flags().contains(PTEFlags::V)
        // (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }

    pub fn is_readable(&self) -> bool {
        self.flags().contains(PTEFlags::R)
        // (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }

    pub fn is_writable(&self) -> bool {
        self.flags().contains(PTEFlags::W)
        // (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }

    pub fn is_executable(&self) -> bool {
        self.flags().contains(PTEFlags::X)
        // (self.flags() & PTEFlags::X) != PTEFlags::empty()
    }
}

/// 每个应用的地址空间都对应一个不同的多级页表，这也就意味这不同页表的起始地址（即页表根节点的 phys 地址）是不一样的;
/// 因此 PageTable 要保存它根节点的物理页号 root_ppn 作为页表唯一的区分标志;
/// 向量 frames 以 FrameTracker 的形式保存了页表所有的节点（包括根节点）所在的物理页帧
/// 将这些 FrameTracker 的生命周期进一步绑定到 PageTable 下面;
/// 当 PageTable 生命周期结束后，向量 frames 里面的那些 FrameTracker 也会被回收，也就意味着存放多级页表节点的那些物理页帧被回收了
pub struct PageTable {
    root_ppn: PhysPageNum,
    frames: Vec<FrameTracker>,
}

impl PageTable {
    /// 通过 new 方法新建一个 PageTable 的时候，它只需有一个根节点
    /// 为此我们需要分配一个物理页帧 FrameTracker 并挂在向量 frames 下，
    /// 然后更新根节点的物理页号 root_ppn
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        PageTable {
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }

    // 通过 vpn 在多级页表(物理内存)中 查找页表项
    fn find_pte_or_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let mut idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;

        for (i, idx) in idxs.iter_mut().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == 2 {
                // 27 bits of vpn (as usize);
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                let frame = frame_alloc().unwrap();
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }
            ppn = pte.ppn();
        }
        result
    }

    fn find_pte(&self, vpn: VirtPageNum) -> Option<&PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&PageTableEntry> = None;

        for (i, idx) in idxs.iter().enumerate() {
            let pte = &ppn.get_pte_array()[*idx];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                return None;
            }
            ppn = pte.ppn();
        }
        result
    }

    // 多级页表并不是被创建出来之后就不再变化的，
    // 为了 MMU 能够通过地址转换正确找到应用地址空间中的数据实际被内核放在内存中位置，
    // 操作系统需要动态维护一个虚拟页号到页表项的映射，支持插入/删除键值对

    #[allow(unused)]
    /// 通过 map 方法来在多级页表中插入一个键值对，注意这里将物理页号 ppn 和页表项标志位 flags 作为不同的参数传入
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_or_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }

    /// 通过 unmap 方法来删除一个键值对，在调用时仅需给出作为索引的虚拟页号
    #[allow(unused)]
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        // let pte = self.find_pte_or_create(vpn).unwrap();
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is invalid before unmmaping", vpn);
        *pte = PageTableEntry::empty();
    }

    /// from_token 可以临时创建一个专用来手动查页表的 PageTable ，
    /// 它仅有一个从传入的 satp token 中得到的多级页表根节点的物理页号，它的 frames 字段为空，也即不实际控制任何资源
    pub fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }

    /// 使用 translate 方法来查找一个虚拟页号对应的页表项
    /// 如果能够找到页表项，那么它会将页表项拷贝一份并返回，否则返回 None
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        // https://rustwiki.org/zh-CN/core/option/enum.Option.html#method.copied
        self.find_pte(vpn).copied()
    }

    // 当遇到需要查一个特定页表（非当前正处在的地址空间的页表时），
    // 便可先通过 PageTable::from_token 新建一个页表，再调用它的 translate 方法查页表

    /// token 会按照 satp CSR 格式要求 构造一个无符号 64 位无符号整数，
    /// 使得其分页模式为 SV39 ，且将当前多级页表的根节点所在的物理页号填充进去。
    /// stap 前 4 个 bit (MODE字段) 设置为 8 时，SV39 分页机制被启用;
    pub fn token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }
}

/// translated_byte_buffer 会以向量的形式返回一组可以在内核空间中直接访问的字节数组切片（注：这个缓冲区的内核虚拟地址范围有可能是不连续的）
/// 将应用地址空间中一个缓冲区转化为在内核空间中能够直接访问的形式。
/// 参数中的 token 是某个应用地址空间的 token ， ptr 和 len 则分别表示该地址空间中的一段缓冲区的起始地址和长度(注：这个缓冲区的应用虚拟地址范围是连续的)。
pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    let page_table = PageTable::from_token(token);
    let mut start = ptr as usize;
    let end = start + len;
    let mut v: Vec<&mut [u8]> = Vec::new();

    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn = start_va.floor();
        let ppn = page_table.translate(vpn).unwrap().ppn();
        vpn.step(); // vpn += 1, next page
        let end_va: VirtAddr = vpn.into();
        if end_va.page_offset() == 0 {
            // end_va is page aligned
            v.push(&mut ppn.get_byte_array()[start_va.page_offset()..]);
        } else {
            v.push(&mut ppn.get_byte_array()[start_va.page_offset()..end_va.page_offset()]);
        }
        start = end_va.into();
    }
    v
}
