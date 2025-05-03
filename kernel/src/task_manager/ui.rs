// use alloc::vec::Vec;
// use noto_sans_mono_bitmap::{get_raster, FontWeight, RasterHeight};
// use pc_keyboard::{layouts::Us104Key, DecodedKey, Keyboard, ScancodeSet2};
// use x86_64::instructions::port::Port;
//
// use crate::scheme::vga::Framebuffer;
//
// pub const FONT_SIZE: RasterHeight = RasterHeight::Size32;
//
// pub struct FramebufferWriter<'a> {
//     framebuffer: &'a mut Framebuffer,
//     buffer: Vec<u8>, // back buffer to avoid tearing
//     dirty_start: (usize, usize),
//     dirty_end: (usize, usize),
// }
//
// #[derive(Debug, Clone, Copy)]
// pub enum Color {
//     Red,
//     Green,
//     Blue,
//     Yellow,
//     Cyan,
//     Magenta,
//     White,
// }
//
// impl Color {
//     fn to_bgra(self) -> [u8; 4] {
//         match self {
//             Color::Red => [0, 0, 255, 255],
//             Color::Green => [0, 255, 0, 255],
//             Color::Blue => [255, 0, 0, 255],
//             Color::Yellow => [0, 255, 255, 255],
//             Color::Cyan => [255, 255, 0, 255],
//             Color::Magenta => [255, 0, 255, 255],
//             Color::White => [255, 255, 255, 255],
//         }
//     }
// }
//
// impl<'a> FramebufferWriter<'a> {
//     pub fn new(framebuffer: &'a mut Framebuffer) -> Self {
//         let buffer = alloc::vec![0; framebuffer.inner.len()];
//         let width = framebuffer.width as usize;
//         let height = framebuffer.height as usize;
//         let mut writer = Self {
//             framebuffer,
//             buffer,
//             dirty_start: (0, 0),
//             dirty_end: (width, height),
//         };
//         writer.clear();
//         writer
//     }
//
//     pub fn clear(&mut self) {
//         let (start_x, start_y) = self.dirty_start;
//         let (end_x, end_y) = self.dirty_end;
//
//         if start_x >= end_x || start_y >= end_y {
//             return;
//         }
//
//         for y in start_y..end_y {
//             let start = y * self.width() + start_x;
//             let end = y * self.width() + end_x;
//             self.buffer[start..end].fill(0);
//         }
//     }
//
//     pub fn width(&self) -> usize {
//         self.framebuffer.width as usize
//     }
//
//     pub fn height(&self) -> usize {
//         self.framebuffer.height as usize
//     }
//
//     pub fn write_text(&mut self, mut x: usize, y: usize, text: &str, color: Color) {
//         for ch in text.chars() {
//             self.write_char(x, y, ch, color);
//             // Advance x after each char
//             if let Some(glyph) = get_raster(ch, FontWeight::Regular, FONT_SIZE) {
//                 x += glyph.width() + 8;
//             } else {
//                 x += 8;
//             }
//         }
//     }
//
//     pub fn draw_line(&mut self, y: usize, thickness: usize, color: Color) {
//         if y >= self.height() {
//             return;
//         }
//
//         let max_rows = self.height() - y;
//         let height = thickness.min(max_rows);
//         let line_bytes = self.framebuffer.pitch as usize;
//         let [b, g, r, a] = color.to_bgra();
//
//         for row in y..(y + height) {
//             let row_offset = row * line_bytes;
//
//             for col in 0..self.width() {
//                 let pixel_offset = row_offset + col * (self.framebuffer.bpp as usize);
//
//                 if pixel_offset + 4 <= self.buffer.len() {
//                     self.buffer[pixel_offset + 0] = b;
//                     self.buffer[pixel_offset + 1] = g;
//                     self.buffer[pixel_offset + 2] = r;
//                     self.buffer[pixel_offset + 3] = a;
//                     self.mark_dirty(col, row);
//                 }
//             }
//         }
//     }
//
//     fn mark_dirty(&mut self, x: usize, y: usize) {
//         self.dirty_start.0 = self.dirty_start.0.min(x);
//         self.dirty_start.1 = self.dirty_start.1.min(y);
//         self.dirty_end.0 = self.dirty_end.0.max(x + 1);
//         self.dirty_end.1 = self.dirty_end.1.max(y + 1);
//     }
//
//     pub fn write_char(&mut self, x: usize, y: usize, ch: char, color: Color) {
//         let glyph = match get_raster(ch, FontWeight::Regular, FONT_SIZE) {
//             Some(g) => g,
//             None => return,
//         };
//
//         let pixels = glyph.raster();
//         let [b, g, r, a] = color.to_bgra();
//         let line_bytes = self.framebuffer.pitch as usize;
//
//         for (row_idx, row) in pixels.iter().enumerate() {
//             let py = y + row_idx;
//             if py >= self.height() {
//                 break;
//             }
//
//             for (col_idx, &intensity) in row.iter().enumerate() {
//                 let px = x + col_idx;
//                 if px >= self.width() {
//                     break;
//                 }
//
//                 if intensity > 0 {
//                     let pixel_offset = py * line_bytes + px * (self.framebuffer.bpp as usize);
//                     if pixel_offset + 4 <= self.buffer.len() {
//                         self.buffer[pixel_offset + 0] = b;
//                         self.buffer[pixel_offset + 1] = g;
//                         self.buffer[pixel_offset + 2] = r;
//                         self.buffer[pixel_offset + 3] = a;
//                         self.mark_dirty(px, py);
//                     }
//                 }
//             }
//         }
//     }
//
//     pub fn flush(&mut self) {
//         let (start_x, start_y) = self.dirty_start;
//         let (end_x, end_y) = self.dirty_end;
//         let bpp = self.framebuffer.bpp as usize;
//         let pitch = self.framebuffer.pitch as usize;
//
//         if start_x >= end_x || start_y >= end_y {
//             return;
//         }
//
//         for y in start_y..end_y {
//             let start = y * pitch + start_x * bpp;
//             let end = y * pitch + end_x * bpp;
//             let back_slice = &self.buffer[start..end];
//             let fb_slice = &mut self.framebuffer.inner[start..end];
//
//             fb_slice.copy_from_slice(back_slice);
//         }
//
//         self.dirty_start = (0, 0);
//         self.dirty_end = (self.width(), self.height());
//     }
// }
//
// pub fn get_key(keyboard: &mut Keyboard<Us104Key, ScancodeSet2>) -> Option<DecodedKey> {
//     let mut status_port = Port::<u8>::new(0x64);
//     let mut port = Port::new(0x60);
//
//     if unsafe { status_port.read() } & 1 == 0 {
//         return None;
//     }
//
//     let scancode: u8 = unsafe { port.read() };
//
//     if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
//         if let Some(key) = keyboard.process_keyevent(key_event) {
//             return Some(key);
//         }
//     }
//
//     None
// }
