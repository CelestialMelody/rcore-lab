use crate::config::KERNEL_HEAP_SIZE;
use buddy_system_allocator::LockedHeap;

#[global_allocator]
/// The global allocator.
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

pub fn init() {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
}

#[alloc_error_handler] // alloc_error_handler is a lang item, see https://doc.rust-lang.org/nightly/core/alloc/trait.GlobalAlloc.html#method.alloc
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Allocation error, layout = {:?}", layout);
}
