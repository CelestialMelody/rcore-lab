//! Implementation of [`MapArea`] and [`MemorySet`].

//! **恒等映射**
//!
//! > 恒等映射的作用范围: 恒等映射方式主要是用在启用多级页表之后，内核仍能够在虚存地址空间中访问一个特定的物理地址指向的物理内存。
//! >
//! > 何时使用了恒等映射: 对于 `map_one` 来说，在虚拟页号 `vpn` 已经确定的情况下，它需要知道要将一个怎么样的页表项插入多级页表。页表项的标志位来源于当前逻辑段的类型为 `MapPermission` 的统一配置，只需将其转换为 `PTEFlags` ；而页表项的物理页号则取决于当前逻辑段映射到物理内存的方式：当以恒等映射 `Identical` 方式映射的时候，物理页号就等于虚拟页号。
//! >
//! > 恒等映射只需被附加到内核地址空间: 应用和内核的地址空间是隔离的，而直接访问物理页帧的操作只会在内核中进行，应用无法看到物理页帧管理器和多级页表等内核数据结构。
//! >
//! > 内核地址空间的低 256GiB 的布局：内核地址空间中需要存在一个恒等映射到内核数据段之外的可用物理页帧的逻辑段，这样才能在启用页表机制之后，内核仍能以纯软件的方式读写这些物理页帧。
//! >
//! > 内核如何访问应用的数据：由于内核要管理应用，所以它负责构建自身和其他应用的多级页表。如果内核获得了一个应用数据的虚地址，内核就可以通过查询应用的页表来把应用的虚地址转换为物理地址，内核直接访问这个地址（注：内核自身的虚实映射是恒等映射），就可以获得应用数据的内容了。

use super::{frame_alloc, FrameTracker};
use super::{PTEFlags, PageTable, PageTableEntry};
use super::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use super::{StepByOne, VPNRange};
use crate::config::{MEMORY_END, PAGE_SIZE, TRAMPOLINE, TRAP_CONTEXT, USER_STACK_SIZE};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use lazy_static::*;
use riscv::paging::MapperFlushGPA;
use riscv::register::satp;
use spin::Mutex;

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn strampoline();
}

/// MapType 描述该逻辑段内的所有虚拟页面映射到物理页帧的同一种方式。
/// 其中 Identical 表示上一节提到的恒等映射方式；
/// 而 Framed 则表示对于每个虚拟页面都有一个新分配的物理页帧与之对应，虚地址与物理地址的映射关系是相对随机的。
/// 恒等映射方式主要是用在启用多级页表之后，内核仍能够在虚存地址空间中访问一个特定的物理地址指向的物理内存。
/// 当逻辑段采用 MapType::Framed 方式映射到物理内存的时候，
/// data_frames 是一个保存了该逻辑段内的每个虚拟页面和它被映射到的物理页帧 FrameTracker 的一个键值对容器 BTreeMap 中，
/// 这些物理页帧被用来存放实际内存数据而不是作为多级页表中的中间节点。
pub enum MapType {
    /// 恒等映射
    Identical,
    /// 随机映射
    Framed,
}

/// bitflags see https://rustwiki.org/zh-CN/rust-cookbook/data_structures/bitfield.html
/// MapPermission 表示控制该逻辑段的访问方式，它是页表项标志位 PTEFlags 的一个子集，
/// 仅保留 U/R/W/X 四个标志位，因为其他的标志位仅与硬件的地址转换机制细节相关，这样的设计能避免引入错误的标志位。
/// PTEflags see http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter4/3sv39-implementation-1.html#id5
bitflags! {
    pub struct MapPermission : u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

/// 以逻辑段 MapArea 为单位描述一段连续地址的虚拟内存。
/// 所谓逻辑段，就是指地址区间中的一段实际可用（即 MMU 通过查多级页表可以正确完成地址转换）的地址连续的虚拟地址区间，
/// 该区间内包含的所有虚拟页面都以一种相同的方式映射到物理页帧，具有可读/可写/可执行等属性。
pub struct MapArea {
    ///  VPNRange 描述一段虚拟页号的连续区间，表示该逻辑段在地址区间中的位置和长度; 是迭代器
    vpn_range: VPNRange,
    /// 当逻辑段采用 MapType::Framed 方式映射到物理内存的时候，
    /// data_frames 是一个保存了该逻辑段内的每个虚拟页面和它被映射到的物理页帧 FrameTracker 的一个键值对容器 BTreeMap 中，
    /// 这些物理页帧被用来存放实际内存数据而不是作为多级页表中的中间节点。
    /// RAII 的思想: 将这些物理页帧的生命周期绑定到它所在的逻辑段 MapArea 下，
    /// 当逻辑段被回收之后这些之前分配的物理页帧也会自动地同时被回收。
    data_frames: BTreeMap<VirtPageNum, FrameTracker>,
    map_type: MapType,
    map_perm: MapPermission,
}

impl MapArea {
    /// new 方法可以新建一个逻辑段结构体，
    /// 传入的起始/终止虚拟地址会分别被下取整/上取整为虚拟页号并传入迭代器 vpn_range 中
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_type: MapType,
        map_perm: MapPermission,
    ) -> Self {
        let start_vpn: VirtPageNum = start_va.floor();
        let end_vpn: VirtPageNum = end_va.ceil();
        Self {
            vpn_range: VPNRange::new(start_vpn, end_vpn),
            data_frames: BTreeMap::new(),
            map_type,
            map_perm,
        }
    }

    // 在虚拟页号 vpn 已经确定的情况下，要将一个怎么样的页表项插入多级页表？
    //
    // 页表项的标志位来源于当前逻辑段的类型为 MapPermission 的统一配置，只需将其转换为 PTEFlags ；
    //
    // 页表项的物理页号则取决于当前逻辑段映射到物理内存的方式：
    // - 当以恒等映射 Identical 方式映射的时候，物理页号就等于虚拟页号；
    // - 当以 Framed 方式映射时，需要分配一个物理页帧让当前的虚拟页面可以映射过去，此时页表项中的物理页号自然就是这个被分配的物理页帧的物理页号。
    // 此时还需要将这个物理页帧 插入到 逻辑段的 data_frames 字段下。
    //
    // 当确定了页表项的标志位和物理页号之后，即可调用多级页表 PageTable 的 map 接口来插入键值对。

    pub fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        let ppn: PhysPageNum;
        match self.map_type {
            MapType::Identical => {
                ppn = PhysPageNum(vpn.0);
            }
            MapType::Framed => {
                let frame = frame_alloc().expect("out of memory");
                ppn = frame.ppn;
                self.data_frames.insert(vpn, frame);
            }
        }
        let pte_flags = PTEFlags::from_bits(self.map_perm.bits).unwrap();
        page_table.map(vpn, ppn, pte_flags);
    }

    pub fn unmap_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        // match self.map_type  {
        //     MapType::Framed => {
        //         self.data_frames.remove(vpn);
        //     }
        //     _ => {}
        // }

        // 当以 Framed 映射的时候，将虚拟页面被映射到的物理页帧 FrameTracker 从 data_frames 中移除，这样这个物理页帧才能立即被回收以备后续分配。
        if self.map_type == MapType::Framed {
            self.data_frames.remove(&vpn)
        }
        page_table.unmap(vpn);
    }

    // map 将当前逻辑段 (到物理内存的映射) 加入到 (传入的该逻辑段所属的地址空间的) 多级页表中。
    // unmap 将当前逻辑段 (到物理内存的映射) 从 (传入的该逻辑段所属的地址空间的) 多级页表中删除。
    // 它们的实现是遍历逻辑段中的所有虚拟页面，并以每个虚拟页面为单位依次在多级页表中进行键值对的插入或删除，
    // 分别对应 MapArea 的 map_one 和 unmap_one 方法。

    /// map 将当前逻辑段 (到物理内存的映射) 加入到 (传入的该逻辑段所属的地址空间的) 多级页表中。
    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.map_one(page_table, vpn);
        }
    }

    /// unmap 将当前逻辑段 (到物理内存的映射) 从 (传入的该逻辑段所属的地址空间的) 多级页表中删除。
    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.unmap_one(page_table, vpn);
        }
    }

    /// copy_data 方法将切片 data 中的数据拷贝到 当前逻辑段 对应的（实际被内核放置在的） 各物理页帧上，从而在地址空间中通过该逻辑段就能访问这些数据。
    pub fn copy_data(&mut self, page_table: &mut PageTable, data: &[u8]) {
        assert_eq!(self.map_type, MapType::Framed);
        let mut start: usize = 0;
        let mut current_vpn = self.vpn_range.get_start();
        let len = data.len();

        // 每个页面的数据拷贝需要确定源 src 和目标 dst 两个切片并直接使用 copy_from_slice 完成复制。
        loop {
            // 调用它的时候需要满足：切片 data 中的数据大小不超过当前逻辑段的总大小，且切片中的数据会被对齐到逻辑段的开头，然后逐页拷贝到实际的物理页帧上。
            let src = &data[start..len.min(start + PAGE_SIZE)];
            // 当确定目标切片 dst 的时候，从传入的当前逻辑段所属的地址空间的多级页表中，手动查找迭代到的虚拟页号被映射到的物理页帧
            // 并通过 get_bytes_array 方法获取该物理页帧的字节数组型可变引用，最后再获取它的切片用于数据拷贝。
            let dst = &mut page_table
                .translate(current_vpn)
                .unwrap()
                .ppn()
                .get_byte_array()[..src.len()];
            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            if start >= len {
                break;
            }
            current_vpn.step();
        }
    }
}

lazy_static! {
    /// 管理内核空间
    pub static ref KERNEL_SPACE: Arc<Mutex<MemorySet>> =
        Arc::new(Mutex::new(MemorySet::new_kernel()));
}

/// MemorySet 控制虚拟内存空间,
/// 它包含了该地址空间的多级页表 page_table 和一个逻辑段 MapArea 的向量 areas 。
/// 这两部分合在一起构成了一个地址空间所需的所有物理页帧。
/// RAII: 当一个地址空间 MemorySet 生命周期结束后，这些物理页帧都会被回收。
pub struct MemorySet {
    /// PageTable 下挂着所有多级页表的节点所在的物理页帧
    page_table: PageTable,
    /// 每个 MapArea 下则挂着对应逻辑段中的数据所在的物理页帧，
    areas: Vec<MemoryArea>,
}

impl MemorySet {
    ///  new_kernel 可以生成内核的地址空间
    ///
    /// from_elf 分析应用的 ELF 文件格式的内容，解析出各数据段并生成对应的地址空间

    /// new_bare 方法可以新建一个空的地址空间
    pub fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }

    /// token 会按照 satp CSR 格式要求 构造一个无符号 64 位无符号整数，
    /// 使得其分页模式为 SV39 ，并将当前多级页表的根节点所在的物理页号填充进去。
    pub fn token(&self) -> usize {
        self.page_table.token()
    }

    /// 将 token 写入当前 CPU 的 satp CSR ，启用 SV39 分页模式，
    /// 并且 MMU 会使用内核地址空间的多级页表进行地址转换。
    pub fn activate(&self) {
        let stap = self.page_table.token();
        unsafe {
            stap::write(stap);
            core::arch::asm!("sfence.vma");
        }
    }

    /// push 方法可以在当前地址空间插入一个新的逻辑段 map_area ，
    /// 如果它是以 Framed 方式映射到物理内存，还可以可选地在那些被映射到的物理页帧上写入一些初始化数据 data；
    /// 需要同时维护地址空间的多级页表 page_table 记录的虚拟页号到页表项的映射关系，
    /// 也需要用到这个映射关系来找到向哪些物理页帧上拷贝初始数据。
    pub fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        map_area.map(&mut self.page_table);
        if let Some(data) = data {
            map_area.copy_data(&mut self.page_table, data);
        }
        self.areas.push(map_area);
    }

    /// insert_framed_area 方法调用 push ，可以在当前地址空间插入一个 Framed 方式映射到物理内存的逻辑段。
    /// 注意该方法的调用者要保证同一地址空间内的任意两个逻辑段不能存在交集。
    pub fn insert_framed_area(
        &mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        permission: MapPermission,
    ) {
        self.push(
            MapArea::new(start_va, end_va, MapType::Framed, permission),
            None,
        );
    }

    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.page_table.translate(vpn)
    }

    // fn map_trampoline()

    // pub fn new_kernel() -> Self {
    //     let mut memory_set = Self::new_bare();
    // }

    // fn from_elf()
}
