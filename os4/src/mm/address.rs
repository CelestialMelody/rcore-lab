//! Implementation of physical and virtual address and page number.

use riscv::{addr::Page, paging::PageTable};

use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS};
use core::fmt::{self, Debug, Formatter};

use super::page_table::PageTableEntry;

#[allow(unused)]
const PA_WIDTH_SV39: usize = 56;
#[allow(unused)]
const VA_WIDTH_SV39: usize = 39;
#[allow(unused)]
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;
#[allow(unused)]
const VPN_WIDTH_SV39: usize = VA_WIDTH_SV39 - PAGE_SIZE_BITS;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
/// Physical address. (44 + 12 = 56 bits)
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
/// Virtual address. (27 + 12 = 39 bits)
pub struct VirtAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
/// Physical page number. (44 bits) 物理页帧
pub struct PhysPageNum(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
/// Virtual page number. (27 bits) 虚拟页面
pub struct VirtPageNum(pub usize);

/// Debugging
impl Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VA:{:#x}", self.0))
    }
}
impl Debug for VirtPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VPN:{:#x}", self.0))
    }
}
impl Debug for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PA:{:#x}", self.0))
    }
}
impl Debug for PhysPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PPN:{:#x}", self.0))
    }
}

/// 类型转换 (Type Conversion)
/// T: {PhysAddr, VirtAddr, PhysPageNum, VirtPageNum}
/// usize -> T: usize.into()
impl From<usize> for PhysAddr {
    fn from(u: usize) -> Self {
        // Self(u & ((1 << PA_WIDTH_SV39) - 1))
        Self(u)
    }
}

impl From<usize> for VirtAddr {
    fn from(u: usize) -> Self {
        // Self(u & ((1 << (VA_WIDTH_SV39 - 1)) - 1))
        Self(u)
    }
}

impl From<usize> for PhysPageNum {
    fn from(u: usize) -> Self {
        // Self(u & ((1 << PPN_WIDTH_SV39) - 1))
        Self(u)
    }
}

impl From<usize> for VirtPageNum {
    fn from(u: usize) -> Self {
        // Self(u & ((1 << (VA_WIDTH_SV39 - PAGE_SIZE_BITS)) - 1))
        Self(u)
    }
}

/// 类型转换 (Type Conversion)
/// T: {PhysAddr, VirtAddr, PhysPageNum, VirtPageNum}
/// T -> usize: T.0
impl From<PhysAddr> for usize {
    fn from(pa: PhysAddr) -> Self {
        pa.0
    }
}

impl From<VirtAddr> for usize {
    fn from(va: VirtAddr) -> Self {
        va.0
    }
}

impl From<PhysPageNum> for usize {
    fn from(ppn: PhysPageNum) -> Self {
        ppn.0
    }
}

impl From<VirtPageNum> for usize {
    fn from(vpn: VirtPageNum) -> Self {
        vpn.0
    }
}

impl PhysAddr {
    /// 向下取整
    pub fn floor(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }

    /// 向上取整
    pub fn ceil(&self) -> PhysPageNum {
        PhysPageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
    }

    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }

    /// whether the address is page aligned
    pub fn is_aligned(&self) -> bool {
        self.0 & (PAGE_SIZE - 1) == 0
    }
}

impl VirtAddr {
    /// 向下取整
    pub fn floor(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }

    /// 向上取整
    pub fn ceil(&self) -> VirtPageNum {
        // 在向上取整时，分子部分要减去1，是为了避免出现，a 能被 b 整除的情况
        VirtPageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
    }

    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }

    /// 是否对齐
    pub fn is_aligned(&self) -> bool {
        self.page_offset() == 0
    }
}

/// 类型转换 (Type Conversion)
/// 地址和页号之间
// 物理地址需要保证它与页面大小对齐才能通过右移转换为物理页号
impl From<PhysAddr> for PhysPageNum {
    fn from(pa: PhysAddr) -> Self {
        // 实际上必须保证页面对齐, 否则 panic
        assert_eq!(pa.page_offset(), 0);
        // pa >> PAGE_SIZE_BITS
        pa.floor()
    }
}

// 从物理页号到物理地址的转换只需左移 12 位
impl From<PhysPageNum> for PhysAddr {
    fn from(ppn: PhysPageNum) -> Self {
        Self(ppn.0 << PAGE_SIZE_BITS)
    }
}

// 虚拟地址需要保证它与页面大小对齐才能通过右移转换为虚拟页号
impl From<VirtAddr> for VirtPageNum {
    fn from(va: VirtAddr) -> Self {
        // 实际上必须保证页面对齐, 否则 panic
        assert_eq!(va.page_offset(), 0);
        // va >> PAGE_SIZE_BITS
        va.floor()
    }
}

// 从虚拟页号到虚拟地址的转换只需左移 12 位
impl From<VirtPageNum> for VirtAddr {
    fn from(vpn: VirtPageNum) -> Self {
        Self(vpn.0 << PAGE_SIZE_BITS)
    }
}

impl VirtPageNum {
    pub fn indexes(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut idx = [0usize; 3];
        // reverse order: idx[2] is the root index
        for i in (0..3).rev() {
            idx[i] = vpn & ((1 << 9) - 1);
            vpn >>= 9;
        }
        idx
    }
}

impl PhysPageNum {
    /// &'static 生命周期针对的仅仅是引用，而不是持有该引用的变量;
    /// 对于变量来说，还是要遵循相应的作用域规则;
    /// 虽然变量被释放，无法再被访问，但是数据依然还会继续存活;
    /// &'static 的引用可以和程序活得一样久;
    /// 猜测: 由于我们的物理地址（真实内存地址）在整个程序运行过程中一直存在, 因此它能跟程序活得一样久，自然它的生命周期是 'static
    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let pa: PhysAddr = (*self).into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut PageTableEntry, 512) }
    }
    pub fn get_byte_array(&self) -> &'static mut [u8] {
        let pa: PhysAddr = (*self).into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut u8, 4096) }
    }
    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: PhysAddr = (*self).into();
        unsafe { (pa.0 as *mut T).as_mut().unwrap() }
    }
}

/// StepByOne
pub trait StepByOne {
    fn step(&mut self);
}

impl StepByOne for VirtPageNum {
    fn step(&mut self) {
        self.0 += 1;
    }
}

/// SimpleRange
#[derive(Clone, Copy)]
pub struct SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    l: T,
    r: T,
}

impl<T> SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    pub fn new(start: T, end: T) -> Self {
        assert!(
            start <= end,
            "start should be less than end; start: {:?}, end: {:?}",
            start,
            end
        );
        Self { l: start, r: end }
    }

    pub fn get_start(&self) -> T {
        self.l
    }

    pub fn get_end(&self) -> T {
        self.r
    }
}

/// Iterator
pub struct SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    current: T,
    end: T,
}

impl<T> SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    pub fn new(l: T, r: T) -> Self {
        Self { current: l, end: r }
    }
}

/// Iterator for SimpleRangeIterator: next
impl<T> Iterator for SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            None
        } else {
            let t = self.current;
            self.current.step();
            Some(t)
        }
    }
}

/// IntoIterator for SimpleRange: into_iter
/// 由于 `into_iter()` 将 `self` 作为值，因此使用 `for` 循环遍历一个集合将消耗该集合。通常，您可能需要迭代一个集合而不使用它。
/// 许多集合提供了在引用上提供迭代器的方法，通常分别称为 `iter()` 和 `iter_mut()`
/// 如果集合类型 `C` 提供 `iter()`，则它通常还为 `&C` 实现 `IntoIterator`，而该实现只是调用 `iter()`。
/// 同样，提供 `iter_mut()` 的集合 `C` 通常通过委派给 `iter_mut()` 来为 `&mut C` 实现 `IntoIterator`。
impl<T> IntoIterator for SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    /// 在 traits 中 type 用于声明 关联类型
    type Item = T;
    type IntoIter = SimpleRangeIterator<T>;
    fn into_iter(self) -> Self::IntoIter {
        SimpleRangeIterator::new(self.l, self.r)
    }
}

/// a simple range structure for virtual page number
pub type VPNRange = SimpleRange<VirtPageNum>;
