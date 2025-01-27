use lazy_static::lazy_static;
use limine::{
    memory_map::Entry,
    request::{HhdmRequest, MemoryMapRequest},
};
use log::debug;

pub mod address;
pub mod loader;
pub mod paging;
pub mod physical;
pub mod stack;

pub const PAGE_SIZE: usize = 4096;

#[used]
#[link_section = ".requests"]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

#[used]
#[link_section = ".requests"]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

lazy_static! {
    pub static ref MEMORY_MAP: &'static [&'static Entry] = {
        debug!("Fetching memory map from bootloader");
        let entries = MEMORY_MAP_REQUEST
            .get_response()
            .expect("Failed to fetch memory map from bootloader")
            .entries();
        debug!("Bootloader reported {} entries", entries.len());

        entries
    };
    pub static ref MEMORY_OFFSET: u64 = {
        debug!("Fetching HHDM from bootloader");
        let offset = HHDM_REQUEST
            .get_response()
            .expect("Failed to fetch HHDM response from bootloader")
            .offset();
        debug!("Bootloader reported {} as HHDM offset", offset);

        offset
    };
}
