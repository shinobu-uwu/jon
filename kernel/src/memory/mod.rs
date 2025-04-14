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
pub mod symbols;

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

#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *s.add(i) = c as u8;
        i += 1;
    }
    s
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *dest.add(i) = *src.add(i);
        i += 1;
    }
    dest
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    let mut i = 0;
    while i < n {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b {
            return a as i32 - b as i32;
        }
        i += 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn bcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    memcmp(s1, s2, n)
}

pub unsafe fn memmove(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if dst == src as *mut u8 || n == 0 {
        return dst;
    }

    if dst < src as *mut u8 {
        for i in 0..n {
            *dst.add(i) = *src.add(i);
        }
    } else {
        for i in (0..n).rev() {
            *dst.add(i) = *src.add(i);
        }
    }

    dst
}
