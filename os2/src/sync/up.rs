use core::cell::{RefCell, RefMut};

/// Wrap a static data structure inside it so that we are
/// able to access it without any `unsafe`.
///
/// We should only use it in uniprocessor.
///
/// In order to get mutable reference of inner data, call
/// `exclusive_access`.

pub struct UnSafeCell<T> {
    inner: RefCell<T>,
}

/// unsafe:
/// 将 UPSafeCell 标记为 Sync 使得它可以作为一个全局变量。
/// 这是 unsafe 行为，因为编译器无法确定我们的 UPSafeCell 能否安全的在多线程间共享。
/// 而我们能够向编译器做出保证，第一个原因是目前我们内核仅运行在单核上，
/// 因此无需在意任何多核引发的数据竞争/同步问题；
/// 第二个原因则是它基于 RefCell 提供了运行时借用检查功能，
/// 从而满足了 Rust 对于借用的基本约束进而保证了内存安全。
unsafe impl<T> Sync for UnSafeCell<T> {}

impl<T> UnSafeCell<T> {
    /// User is responsible to guarantee that inner struct is only used in
    /// uniprocessor.
    /// unsafe:
    /// 希望使用者在创建一个 UPSafeCell 的时候
    /// 保证在访问 UPSafeCell 内包裹的数据的时候始终不违背上述模式：
    /// 即访问之前调用 exclusive_access ，访问之后销毁借用标记再进行下一次访问。
    /// 这只能依靠使用者自己来保证，但我们提供了一个保底措施：
    /// 当使用者违背了上述模式，比如访问之后忘记销毁就开启下一次访问时，程序会 panic 并退出
    pub unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }
    /// Panic if the data has been borrowed.
    /// 调用 exclusive_access 可以得到它包裹的数据的独占访问权
    /// 当我们要访问数据时（无论读还是写），需要首先调用 exclusive_access 获得数据的可变借用标记，
    /// 通过它可以完成数据的读写，在操作完成之后我们需要销毁这个标记，此后才能开始对该数据的下一次访问。
    /// 相比 RefCell 它不再允许多个读操作同时存在
    pub fn exclusive_access(&self) -> RefMut<'_, T> {
        // `<'_, T>`
        self.inner.borrow_mut()
    }
}
