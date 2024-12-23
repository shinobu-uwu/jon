pub mod allocator;
pub mod paging;
pub mod physical;

pub(super) fn init() {
    allocator::init();
}
