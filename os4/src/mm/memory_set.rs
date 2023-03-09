//! Implementation of [`MapArea`] and [`MemorySet`].

use super::{frame_alloc, FrameTracker};
use super::{PTEFlags, PageTable, PageTableEntry};
use super::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use super::{StepByOne, VPNRange};
use crate::config::{MEMORY_END, PAGE_SIZE, TRAMPOLINE, TRAP_CONTEXT, USER_STACK_SIZE};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use lazy_static::*;
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

#[derive(Copy, Clone, PartialEq, Debug)]
/// map type for memory set: identical or framed
pub enum MapType {
    /// 恒等映射
    Identical,
    /// 随机映射
    Framed,
}

bitflags! {
    /// map permission corresponding to that in pte: `R W X U`
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}
lazy_static! {
    /// a memory set instance through lazy_static! managing kernel space
    pub static ref KERNEL_SPACE: Arc<Mutex<MemorySet>> =
        Arc::new(Mutex::new(MemorySet::new_kernel()));
}

/// map area structure, controls a contiguous piece of virtual memory
pub struct MapArea {
    vpn_range: VPNRange,
    /// 当逻辑段采用 MapType::Framed 方式映射到物理内存的时候，
    /// data_frames 是一个保存了该逻辑段内的每个虚拟页面和它被映射到的物理页帧 FrameTracker 的一个键值对容器 BTreeMap 中，
    /// 这些物理页帧被用来存放实际内存数据而不是作为多级页表中的中间节点。
    data_frames: BTreeMap<VirtPageNum, FrameTracker>,
    map_type: MapType,
    map_perm: MapPermission,
}

impl MapArea {
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_type: MapType,
        map_perm: MapPermission,
    ) -> Self {
        // 传入的起始/终止虚拟地址会分别被下取整/上取整为虚拟页号并传入迭代器 vpn_range 中
        let start_vpn: VirtPageNum = start_va.floor();
        let end_vpn: VirtPageNum = end_va.ceil();
        Self {
            vpn_range: VPNRange::new(start_vpn, end_vpn),
            data_frames: BTreeMap::new(),
            map_type,
            map_perm,
        }
    }
    pub fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        let ppn: PhysPageNum;

        // 页表项的物理页号则取决于当前逻辑段映射到物理内存的方式：
        // 1. 当以恒等映射 Identical 方式映射的时候，物理页号就等于虚拟页号；
        // 2. 当以 Framed 方式映射时，需要分配一个物理页帧让当前的虚拟页面可以映射过去，此时页表项中的物理页号自然就是这个被分配的物理页帧的物理页号。
        //    此时还需要将这个物理页帧 插入到 逻辑段的 data_frames 字段下，维护地址空间的多级页表 page_table 记录的虚拟页号到页表项的映射关系。
        //    可以用这个映射关系来找到向哪些物理页帧上拷贝初始数据 (fn copy_data)
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
        // 页表项的标志位来源于当前逻辑段的类型为 MapPermission 的统一配置，只需将其转换为 PTEFlags ；
        let pte_flags = PTEFlags::from_bits(self.map_perm.bits).unwrap();
        page_table.map(vpn, ppn, pte_flags);
    }
    #[allow(unused)]
    pub fn unmap_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        // 当以 Framed 映射的时候，将虚拟页面被映射到的物理页帧 FrameTracker 从 data_frames 中移除，这样这个物理页帧才能立即被回收以备后续分配。
        if self.map_type == MapType::Framed {
            self.data_frames.remove(&vpn);
        }
        page_table.unmap(vpn);
    }
    /// map 将当前逻辑段 (到物理内存的映射) 加入到 (传入的该逻辑段所属的地址空间的) 多级页表中。
    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.map_one(page_table, vpn);
        }
    }
    #[allow(unused)]
    /// unmap 将当前逻辑段 (到物理内存的映射) 从 (传入的该逻辑段所属的地址空间的) 多级页表中删除。
    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.unmap_one(page_table, vpn);
        }
    }
    /// copy_data 方法将切片 data 中的数据拷贝到 当前逻辑段 对应的（实际被内核放置在的） 各物理页帧上，从而在地址空间中通过该逻辑段就能访问这些数据。
    /// data: start-aligned but maybe with shorter length.
    /// assume that all frames were cleared before
    pub fn copy_data(&mut self, page_table: &mut PageTable, data: &[u8]) {
        assert_eq!(self.map_type, MapType::Framed);
        let mut start: usize = 0;
        let mut current_vpn = self.vpn_range.get_start();
        let len = data.len();

        // 每个页面的数据拷贝需要确定源 src 和目标 dst 两个切片并直接使用 copy_from_slice 完成复制
        loop {
            // 调用它的时候需要满足：切片 data 中的数据大小不超过当前逻辑段的总大小，且切片中的数据会被对齐到逻辑段的开头，然后逐页拷贝到实际的物理页帧上
            let src = &data[start..len.min(start + PAGE_SIZE)];
            // 当确定目标切片 dst 的时候，从传入的当前逻辑段所属的地址空间的多级页表中，手动查找迭代到的虚拟页号被映射到的物理页帧
            // 并通过 get_bytes_array 方法获取该物理页帧的字节数组型可变引用，最后再获取它的切片用于数据拷贝。
            let dst = &mut page_table
                .translate(current_vpn)
                .unwrap()
                .ppn()
                .get_bytes_array()[..src.len()];

            dst.copy_from_slice(src);

            start += PAGE_SIZE;
            if start >= len {
                break;
            }
            current_vpn.step();
        }
    }
}

/// memory set structure, controls virtual-memory space.
/// MemorySet 控制虚拟内存空间,
/// 它包含了该地址空间的多级页表 page_table 和一个逻辑段 MapArea 的向量 areas 。
/// 这两部分合在一起构成了一个地址空间所需的所有物理页帧。
pub struct MemorySet {
    /// PageTable 下挂着所有多级页表的节点所在的物理页帧
    page_table: PageTable,
    /// 每个 MapArea 下则挂着对应逻辑段中的数据所在的物理页帧，
    areas: Vec<MapArea>,
}

impl MemorySet {
    /// new_bare 方法可以新建一个空的地址空间
    pub fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }
    /// 内核页表的起始物理地址
    /// token 会按照 satp CSR 格式要求 构造一个无符号 64 位无符号整数，
    /// 使得其分页模式为 SV39 ，并将当前多级页表的根节点所在的物理页号填充进去。
    pub fn token(&self) -> usize {
        self.page_table.token()
    }
    /// 将 token 写入当前 CPU 的 satp CSR ，启用 SV39 分页模式，
    /// 并且 MMU 会使用内核地址空间的多级页表进行地址转换。
    pub fn activate(&self) {
        let satp = self.page_table.token();
        unsafe {
            satp::write(satp);
            // 一旦我们修改 satp 就会切换地址空间，快表中的键值对就会失效（因为快表保存着老地址空间的映射关系，切换到新地址空间后，老的映射关系就没用了）。
            // 为了确保 MMU 的地址转换能够及时与 satp 的修改同步，我们需要立即使用 sfence.vma 指令将快表清空，这样 MMU 就不会看到快表中已经过期的键值对了。
            core::arch::asm!("sfence.vma");
        }
    }
    /// 从当前地址空间的多级页表中查找虚拟页号 vpn 对应的页表项
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.page_table.translate(vpn)
    }
    /// Assume that no conflicts.
    /// 在当前地址空间插入一个 Framed 方式映射到物理内存的逻辑段。
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
    /// 在当前地址空间插入一个新的逻辑段 map_area
    fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        map_area.map(&mut self.page_table);
        if let Some(data) = data {
            map_area.copy_data(&mut self.page_table, data);
        }
        self.areas.push(map_area);
    }

    pub fn remove_framed_area(&mut self, start_va: VirtAddr, end_va: VirtAddr) {
        self.remove(start_va, end_va);
    }

    fn remove(&mut self, start_va: VirtAddr, end_va: VirtAddr) {
        let start_vpn = start_va.floor();
        let end_vpn = end_va.ceil();
        let vpn_range: VPNRange = VPNRange::new(start_vpn, end_vpn);
        for vpn in vpn_range {
            self.page_table.unmap(vpn);
        }
    }
    /// Mention that trampoline is not collected by areas.
    /// 将内核的 trampoline 代码段映射到虚拟地址 TRAMPOLINE 上.
    /// 为了实现方便并没有新增逻辑段 MemoryArea 而是直接在多级页表中插入一个从地址空间的最高虚拟页面映射到跳板汇编代码所在的物理页帧的键值对，
    /// 访问权限与代码段相同，即 RX （可读可执行）。
    fn map_trampoline(&mut self) {
        self.page_table.map(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysAddr::from(strampoline as usize).into(),
            PTEFlags::R | PTEFlags::X,
        );
    }
    /// new_kernel 可以生成内核的地址空间
    pub fn new_kernel() -> Self {
        let mut memory_set = Self::new_bare();

        // map trampoline
        memory_set.map_trampoline();

        // map kernel sections
        // 内核的四个逻辑段 .text/.rodata/.data/.bss 被恒等映射到物理内存，
        // 这使得我们在无需调整内核内存布局 os/src/linker.ld 的情况下就仍能象启用页表机制之前那样访问内核的各个段。

        // 从 os/src/linker.ld 中引用了很多表示各个段位置的符号，
        // 而后在 new_kernel 中，我们从低地址到高地址依次创建 5 个逻辑段并通过 push 方法将它们插入到内核地址空间中
        info!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        info!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        info!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        info!(
            ".bss [{:#x}, {:#x})",
            sbss_with_stack as usize, ebss as usize
        );
        info!(
            "physical memory: [{:#x}, {:#x})",
            ekernel as usize, MEMORY_END
        );

        // 借用页表机制对这些逻辑段的访问方式做出了限制，这都是为了在硬件的帮助下能够尽可能发现内核中的 bug;
        // 四个逻辑段的 U 标志位均未被设置，使得 CPU 只能在处于 S 特权级（或以上）时访问它们；

        info!("mapping .text section");
        memory_set.push(
            MapArea::new(
                (stext as usize).into(),
                (etext as usize).into(),
                MapType::Identical,
                // 代码段 .text 不允许被修改；
                MapPermission::R | MapPermission::X,
            ),
            None,
        );
        info!("mapping .rodata section");
        memory_set.push(
            MapArea::new(
                (srodata as usize).into(),
                (erodata as usize).into(),
                MapType::Identical,
                // 只读数据段 .rodata 不允许被修改，也不允许从它上面取指执行；
                MapPermission::R,
            ),
            None,
        );
        info!("mapping .data section");
        memory_set.push(
            MapArea::new(
                (sdata as usize).into(),
                (edata as usize).into(),
                MapType::Identical,
                // .bss 允许被读写，但是不允许从它上面取指执行。
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        info!("mapping .bss section");
        memory_set.push(
            MapArea::new(
                (sbss_with_stack as usize).into(),
                (ebss as usize).into(),
                MapType::Identical,
                // .bss 允许被读写，但是不允许从它上面取指执行。
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        // 内核地址空间中需要存在一个恒等映射到内核数据段之外的可用物理页帧的逻辑段，
        // 这样才能在启用页表机制之后，内核仍能以纯软件的方式读写这些物理页帧
        info!("mapping physical memory");
        memory_set.push(
            MapArea::new(
                (ekernel as usize).into(),
                MEMORY_END.into(),
                MapType::Identical,
                // 标志位仅包含 rw ，意味着该逻辑段只能在 S 特权级以上访问，并且只能读写。
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        memory_set
    }
    /// Include sections in elf and trampoline and TrapContext and user stack,
    /// also returns user_sp and entry point.
    /// from_elf 分析应用的 ELF 文件格式的内容，解析出各数据段并生成对应的地址空间。
    /// 返回应用地址空间 memory_set 、用户栈虚拟地址 user_stack_top 以及从解析 ELF 得到的该应用入口点地址，它们将被我们用来创建应用的任务控制块。
    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        let mut memory_set = Self::new_bare();

        // map_trampoline 会将内核的 trampoline 代码段映射到内核地址空间的最高处
        memory_set.map_trampoline();

        // map program headers of elf, with U flag
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();

        let elf_header = elf.header;

        let magic = elf_header.pt1.magic;
        // 检查 elf 文件是否为正确的 elf 文件 (magic number: see https://en.wikipedia.org/wiki/Executable_and_Linkable_Format)
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");

        // 获取 program header 的数量
        let ph_count = elf_header.pt2.ph_count();

        let mut max_end_vpn = VirtPageNum(0);

        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();

            // program header 的类型是 LOAD ，这表明它有被内核加载的必要
            // 此时不必理会其他类型的 program header
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                // 通过 ph.virtual_addr() 和 ph.mem_size() 来计算这一区域在应用地址空间中的位置
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();

                let mut map_perm = MapPermission::U;

                // 通过 ph.flags() 来确认这一区域访问方式的限制并将其转换为 MapPermission 类型（注意它默认包含 U 标志位）
                let ph_flags = ph.flags();

                if ph_flags.is_read() {
                    map_perm |= MapPermission::R;
                }
                if ph_flags.is_write() {
                    map_perm |= MapPermission::W;
                }
                if ph_flags.is_execute() {
                    map_perm |= MapPermission::X;
                }

                let map_area = MapArea::new(start_va, end_va, MapType::Framed, map_perm);

                max_end_vpn = map_area.vpn_range.get_end();

                memory_set.push(
                    map_area,
                    // 当前 program header 数据被存放的位置可以通过 ph.offset() 和 ph.file_size() 来找到
                    // 这里不使用 ph.mem_size 而是 ph.file_size 的原因：
                    // 当存在一部分零初始化的时候， ph.file_size() 将会小于 ph.mem_size() ，因为这些零出于缩减可执行文件大小的原因不应该实际出现在 ELF 数据中
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),
                );
            }
        }
        // map user stack with U flags
        // 在前面加载各个 program header 的时候，我们就已经维护了 max_end_vpn 记录目前涉及到的最大的虚拟页号，只需紧接着在它上面再放置一个保护页面和用户栈即可。
        // vpn -> va -> usize
        let max_end_va: VirtAddr = max_end_vpn.into();
        let mut user_stack_bottom: usize = max_end_va.into();

        // guard page
        user_stack_bottom += PAGE_SIZE;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        memory_set.push(
            MapArea::new(
                user_stack_bottom.into(),
                user_stack_top.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W | MapPermission::U,
            ),
            None,
        );

        // map TrapContext
        // 在应用地址空间中映射次高页面来存放 Trap 上下文
        memory_set.push(
            MapArea::new(
                TRAP_CONTEXT.into(),
                TRAMPOLINE.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );

        // 返回应用地址空间 memory_set 、用户栈虚拟地址 user_stack_top 以及从解析 ELF 得到的该应用入口点地址，它们将被我们用来创建应用的任务控制块。
        (
            memory_set,
            user_stack_top,
            elf.header.pt2.entry_point() as usize,
        )
    }
}

#[allow(unused)]
pub fn remap_test() {
    let mut kernel_space = KERNEL_SPACE.lock();
    let mid_text: VirtAddr = ((stext as usize + etext as usize) / 2).into();
    let mid_rodata: VirtAddr = ((srodata as usize + erodata as usize) / 2).into();
    let mid_data: VirtAddr = ((sdata as usize + edata as usize) / 2).into();
    assert!(!kernel_space
        .page_table
        .translate(mid_text.floor())
        .unwrap()
        .writable());
    assert!(!kernel_space
        .page_table
        .translate(mid_rodata.floor())
        .unwrap()
        .writable());
    assert!(!kernel_space
        .page_table
        .translate(mid_data.floor())
        .unwrap()
        .executable());
    info!("remap_test passed!");
}

// 恒等映射
//
// 恒等映射的作用范围: 恒等映射方式主要是用在启用多级页表之后，内核仍能够在虚存地址空间中访问一个特定的物理地址指向的物理内存。
// 何时使用了恒等映射: 对于 `map_one` 来说，在虚拟页号 `vpn` 已经确定的情况下，它需要知道要将一个怎么样的页表项插入多级页表。页表项的标志位来源于当前逻辑段的类型为 `MapPermission` 的统一配置，只需将其转换为 `PTEFlags` ；而页表项的物理页号则取决于当前逻辑段映射到物理内存的方式：当以恒等映射 `Identical` 方式映射的时候，物理页号就等于虚拟页号。
// 恒等映射只需被附加到内核地址空间: 应用和内核的地址空间是隔离的，而直接访问物理页帧的操作只会在内核中进行，应用无法看到物理页帧管理器和多级页表等内核数据结构。
// 内核地址空间的低 256GiB 的布局：内核地址空间中需要存在一个恒等映射到内核数据段之外的可用物理页帧的逻辑段，这样才能在启用页表机制之后，内核仍能以纯软件的方式读写这些物理页帧。
// 内核如何访问应用的数据：由于内核要管理应用，所以它负责构建自身和其他应用的多级页表。如果内核获得了一个应用数据的虚地址，内核就可以通过查询应用的页表来把应用的虚地址转换为物理地址，内核直接访问这个地址（注：内核自身的虚实映射是恒等映射），就可以获得应用数据的内容了。

// MapType 描述该逻辑段内的所有虚拟页面映射到物理页帧的同一种方式。
// 其中 Identical 表示上一节提到的恒等映射方式；
// 而 Framed 则表示对于每个虚拟页面都有一个新分配的物理页帧与之对应，虚地址与物理地址的映射关系是相对随机的。
// 恒等映射方式主要是用在启用多级页表之后，内核仍能够在虚存地址空间中访问一个特定的物理地址指向的物理内存。
// 当逻辑段采用 MapType::Framed 方式映射到物理内存的时候，
// data_frames 是一个保存了该逻辑段内的每个虚拟页面和它被映射到的物理页帧 FrameTracker 的一个键值对容器 BTreeMap 中，
// 这些物理页帧被用来存放实际内存数据而不是作为多级页表中的中间节点。

// bitflags see https://rustwiki.org/zh-CN/rust-cookbook/data_structures/bitfield.html
// MapPermission 表示控制该逻辑段的访问方式，它是页表项标志位 PTEFlags 的一个子集，
// 仅保留 U/R/W/X 四个标志位，因为其他的标志位仅与硬件的地址转换机制细节相关，这样的设计能避免引入错误的标志位。
// PTEflags see http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter4/3sv39-implementation-1.html#id5

// 当逻辑段采用 MapType::Framed 方式映射到物理内存的时候，
// data_frames 是一个保存了该逻辑段内的每个虚拟页面和它被映射到的物理页帧 FrameTracker 的一个键值对容器 BTreeMap 中，
// 这些物理页帧被用来存放实际内存数据而不是作为多级页表中的中间节点。
// RAII 的思想: 将这些物理页帧的生命周期绑定到它所在的逻辑段 MapArea 下，
// 当逻辑段被回收之后这些之前分配的物理页帧也会自动地同时被回收。

// 在虚拟页号 vpn 已经确定的情况下，要将一个怎么样的页表项插入多级页表？
// 页表项的标志位来源于当前逻辑段的类型为 MapPermission 的统一配置，只需将其转换为 PTEFlags；
// 页表项的物理页号则取决于当前逻辑段映射到物理内存的方式：
// - 当以恒等映射 Identical 方式映射的时候，物理页号就等于虚拟页号；
// - 当以 Framed 方式映射时，需要分配一个物理页帧让当前的虚拟页面可以映射过去，此时页表项中的物理页号自然就是这个被分配的物理页帧的物理页号。
// 此时还需要将这个物理页帧 插入到 逻辑段的 data_frames 字段下。
// 当确定了页表项的标志位和物理页号之后，即可调用多级页表 PageTable 的 map 接口来插入键值对。

// MapArea:
// map 将当前逻辑段 (到物理内存的映射) 加入到 (传入的该逻辑段所属的地址空间的) 多级页表中。
// unmap 将当前逻辑段 (到物理内存的映射) 从 (传入的该逻辑段所属的地址空间的) 多级页表中删除。
// 它们的实现是遍历逻辑段中的所有虚拟页面，并以每个虚拟页面为单位依次在多级页表中进行键值对的插入或删除，
// 分别对应 MapArea 的 map_one 和 unmap_one 方法。

// map trampoline: page_table.map
// else memory_set.push(map_area.new, data) -> map_area.map -> map_area.map_one -> page_table.map
