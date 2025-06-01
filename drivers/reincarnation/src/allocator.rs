use buddy_system_allocator::LockedHeap;
use jon_common::syscall::task::brk;

#[global_allocator]
pub static GLOBAL_ALLOC: LockedHeap<32> = LockedHeap::empty();

pub fn init() {
    const HEAP_SIZE: usize = 3 * 1024 * 1024; // 3 MiB
    let heap_start = match brk(HEAP_SIZE) {
        Ok(addr) => addr,
        Err(_) => panic!("Failed to allocate heap"),
    };

    unsafe {
        GLOBAL_ALLOC.lock().init(heap_start, HEAP_SIZE);
    }
}
