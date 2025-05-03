use alloc::vec::Vec;
use noto_sans_mono_bitmap::{FontWeight, RasterHeight, get_raster};

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

pub struct FramebufferWriter {
    fd: usize,
    framebuffer: Framebuffer,
    buffer: Vec<u8>,
    dirty_start: (usize, usize),
    dirty_end: (usize, usize),
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
    fn to_bgra(self) -> [u8; 4] {
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

impl FramebufferWriter {
    pub fn new(fd: usize, framebuffer: Framebuffer) -> Self {
        let buffer = alloc::vec![0; (framebuffer.height * framebuffer.pitch) as usize];
        let width = framebuffer.width as usize;
        let height = framebuffer.height as usize;
        let mut writer = Self {
            fd,
            framebuffer,
            buffer,
            dirty_start: (0, 0),
            dirty_end: (width, height),
        };
        writer.clear();
        writer
    }

    pub fn clear(&mut self) {
        let (start_x, start_y) = self.dirty_start;
        let (end_x, end_y) = self.dirty_end;

        if start_x >= end_x || start_y >= end_y {
            return;
        }

        for y in start_y..end_y {
            let start = y * self.width() + start_x;
            let end = y * self.width() + end_x;
            self.buffer[start..end].fill(0);
        }
    }

    pub fn width(&self) -> usize {
        self.framebuffer.width as usize
    }

    pub fn height(&self) -> usize {
        self.framebuffer.height as usize
    }

    pub fn write_text(&mut self, mut x: usize, y: usize, text: &str, color: Color) {
        for ch in text.chars() {
            self.write_char(x, y, ch, color);
            // Advance x after each char
            if let Some(glyph) = get_raster(ch, FontWeight::Regular, FONT_SIZE) {
                x += glyph.width() + 8;
            } else {
                x += 8;
            }
        }
    }

    pub fn draw_line(&mut self, y: usize, thickness: usize, color: Color) {
        if y >= self.height() {
            return;
        }

        let max_rows = self.height() - y;
        let height = thickness.min(max_rows);
        let line_bytes = self.framebuffer.pitch as usize;
        let [b, g, r, a] = color.to_bgra();

        for row in y..(y + height) {
            let row_offset = row * line_bytes;

            for col in 0..self.width() {
                let pixel_offset = row_offset + col * (self.framebuffer.bpp as usize);

                if pixel_offset + 4 <= self.buffer.len() {
                    self.buffer[pixel_offset + 0] = b;
                    self.buffer[pixel_offset + 1] = g;
                    self.buffer[pixel_offset + 2] = r;
                    self.buffer[pixel_offset + 3] = a;
                    self.mark_dirty(col, row);
                }
            }
        }
    }

    fn mark_dirty(&mut self, x: usize, y: usize) {
        self.dirty_start.0 = self.dirty_start.0.min(x);
        self.dirty_start.1 = self.dirty_start.1.min(y);
        self.dirty_end.0 = self.dirty_end.0.max(x + 1);
        self.dirty_end.1 = self.dirty_end.1.max(y + 1);
    }

    pub fn write_char(&mut self, x: usize, y: usize, ch: char, color: Color) {
        let glyph = match get_raster(ch, FontWeight::Regular, FONT_SIZE) {
            Some(g) => g,
            None => return,
        };

        let pixels = glyph.raster();
        let [b, g, r, a] = color.to_bgra();
        let line_bytes = self.framebuffer.pitch as usize;

        for (row_idx, row) in pixels.iter().enumerate() {
            let py = y + row_idx;
            if py >= self.height() {
                break;
            }

            for (col_idx, &intensity) in row.iter().enumerate() {
                let px = x + col_idx;
                if px >= self.width() {
                    break;
                }

                if intensity > 0 {
                    let pixel_offset = py * line_bytes + px * (self.framebuffer.bpp as usize);
                    if pixel_offset + 4 <= self.buffer.len() {
                        self.buffer[pixel_offset + 0] = b;
                        self.buffer[pixel_offset + 1] = g;
                        self.buffer[pixel_offset + 2] = r;
                        self.buffer[pixel_offset + 3] = a;
                        self.mark_dirty(px, py);
                    }
                }
            }
        }
    }

    pub fn flush(&mut self) {
        jon_common::syscall::fs::write(self.fd, &self.buffer).unwrap();
        // let (start_x, start_y) = self.dirty_start;
        // let (end_x, end_y) = self.dirty_end;
        // let bpp = self.framebuffer.bpp as usize;
        // let pitch = self.framebuffer.pitch as usize;
        //
        // if start_x >= end_x || start_y >= end_y {
        //     return;
        // }
        //
        // for y in start_y..end_y {
        //     let start = y * pitch + start_x * bpp;
        //     let end = y * pitch + end_x * bpp;
        //     let back_slice = &self.buffer[start..end];
        // }
        //
        // self.dirty_start = (0, 0);
        // self.dirty_end = (self.width(), self.height());
    }
}
