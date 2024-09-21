use dummy::Dummy;

pub mod dummy;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

#[global_allocator]
static ALLOCATOR: Dummy = unsafe {
    let alloc = Dummy::new();
    alloc.init(HEAP_START, HEAP_SIZE);
    alloc
};
