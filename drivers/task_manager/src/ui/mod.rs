use crate::{proc::NEW_PROCS, writer::FramebufferWriter};
use core::ffi::CStr;

use alloc::format;
use noto_sans_mono_bitmap::RasterHeight;

use crate::{
    Y_OFFSET,
    proc::{Proc, State},
};

pub mod screen;

pub const FONT_SIZE: RasterHeight = RasterHeight::Size32;
pub const WIDHTH: u64 = 1280;
pub const HEIGHT: u64 = 720;
pub const BPP: u16 = 4;
pub const PITCH: u64 = 5120;

#[derive(Debug)]
pub struct Framebuffer {
    pub width: u64,
    pub height: u64,
    pub bpp: u16,
    pub pitch: u64,
}

impl Default for Framebuffer {
    fn default() -> Self {
        Self {
            width: WIDHTH,
            height: HEIGHT,
            bpp: BPP,
            pitch: PITCH,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
    White,
}

impl Color {
    pub fn to_bgra(self) -> [u8; 4] {
        match self {
            Color::Red => [0, 0, 255, 255],
            Color::Green => [0, 255, 0, 255],
            Color::Blue => [255, 0, 0, 255],
            Color::Yellow => [0, 255, 255, 255],
            Color::Cyan => [255, 255, 0, 255],
            Color::Magenta => [255, 0, 255, 255],
            Color::White => [255, 255, 255, 255],
        }
    }
}
