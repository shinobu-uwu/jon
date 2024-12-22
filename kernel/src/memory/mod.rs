#[used]
#[link_section = ".requests"]
pub static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

use limine::request::MemoryMapRequest;

pub mod address;
pub mod paging;
pub mod physical;
