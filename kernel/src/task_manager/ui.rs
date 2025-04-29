use alloc::vec::Vec;
use noto_sans_mono_bitmap::{get_raster, FontWeight, RasterHeight};
use pc_keyboard::{
    layouts::{self, Us104Key},
    DecodedKey, HandleControl, Keyboard, ScancodeSet1,
};
use x86_64::instructions::port::Port;

use crate::scheme::{ps2::CONTROLLER, vga::Framebuffer};

const SIZE: RasterHeight = RasterHeight::Size32;

pub struct FramebufferWriter<'a> {
    framebuffer: &'a mut Framebuffer,
    buffer: Vec<u8>, // back buffer to avoid tearing
}

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
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
        }
    }
}

impl<'a> FramebufferWriter<'a> {
    pub fn new(framebuffer: &'a mut Framebuffer) -> Self {
        let buffer = alloc::vec![0; framebuffer.inner.len()];
        let mut writer = Self {
            framebuffer,
            buffer,
        };
        writer.clear();
        writer
    }

    pub fn clear(&mut self) {
        self.buffer.fill(0);
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
            if let Some(glyph) = get_raster(ch, FontWeight::Regular, SIZE) {
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
                }
            }
        }
    }

    pub fn write_char(&mut self, x: usize, y: usize, ch: char, color: Color) {
        let glyph = match get_raster(ch, FontWeight::Regular, SIZE) {
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
                    }
                }
            }
        }
    }

    pub fn flush(&mut self) {
        self.framebuffer.inner.copy_from_slice(&self.buffer);
    }
}

pub fn get_key(keyboard: &mut Keyboard<Us104Key, ScancodeSet1>) -> Option<DecodedKey> {
    let mut status_port = Port::<u8>::new(0x64);
    let mut port = Port::new(0x60);

    if unsafe { status_port.read() } & 1 == 0 {
        return None;
    }

    let scancode: u8 = unsafe { port.read() };

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            return Some(key);
        }
    }

    None
}
