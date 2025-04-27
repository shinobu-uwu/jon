use jon_common::syscall::fs::{lseek, write};
use noto_sans_mono_bitmap::{FontWeight, RasterHeight, get_raster};

pub const FRAMEBUFFER_WIDTH: usize = 640;
pub const FRAMEBUFFER_HEIGHT: usize = 480;
pub const FRAMEBUFFER_BPP: usize = 4;
pub const BUF_SIZE: usize = 0x4000;

/// Draws a horizontal line of solid `color`, `thickness` pixels tall,
/// starting at vertical coordinate `y`.
pub fn draw_line(fd: usize, y: usize, thickness: usize, color: Color) {
    if y >= FRAMEBUFFER_HEIGHT {
        return;
    }

    let max_rows = FRAMEBUFFER_HEIGHT - y;
    let height = thickness.min(max_rows);
    let line_bytes = FRAMEBUFFER_WIDTH * FRAMEBUFFER_BPP;

    let mut buf = [0u8; BUF_SIZE];
    for pixel in buf.chunks_exact_mut(FRAMEBUFFER_BPP) {
        let color_bytes = match color {
            // BGRA format instead of RGBA
            Color::Red => [0, 0, 255, 255],
            Color::Green => [0, 255, 0, 255],
            Color::Blue => [255, 0, 0, 255],
            Color::Yellow => [0, 255, 255, 255],
            Color::Cyan => [255, 255, 0, 255],
            Color::Magenta => [255, 0, 255, 255],
        };

        pixel.copy_from_slice(&color_bytes);
    }

    for row in y..(y + height) {
        let offset = row * line_bytes;

        lseek(fd, offset, 0).unwrap();

        let mut written = 0;
        while written < line_bytes {
            let to_write = (line_bytes - written).min(BUF_SIZE);
            let n = write(fd, &buf[..to_write]).unwrap();

            if n == 0 {
                break;
            }

            written += n as usize;
        }
    }
}

pub fn draw_text(fd: usize, mut x: usize, y: usize, text: &str, color: Color) {
    for ch in text.chars() {
        let glyph = match get_raster(ch, FontWeight::Regular, RasterHeight::Size16) {
            Some(g) => g,
            None => continue,
        };

        draw_char(fd, x, y, ch, color);
        x += glyph.width() + 8;
    }
}

pub fn draw_char(fd: usize, x: usize, y: usize, ch: char, color: Color) {
    let glyph = match get_raster(ch, FontWeight::Regular, RasterHeight::Size32) {
        Some(g) => g,
        None => return,
    };
    let pixels: &[&[u8]] = glyph.raster();

    let [b, g, r, a] = match color {
        Color::Red => [0, 0, 255, 255],
        Color::Green => [0, 255, 0, 255],
        Color::Blue => [255, 0, 0, 255],
        Color::Yellow => [0, 255, 255, 255],
        Color::Cyan => [255, 255, 0, 255],
        Color::Magenta => [255, 0, 255, 255],
    };

    let line_bytes = FRAMEBUFFER_WIDTH * FRAMEBUFFER_BPP;

    for row in 0..glyph.height() {
        let screen_y = y + row;
        if screen_y >= FRAMEBUFFER_HEIGHT {
            break;
        }

        let row_offset = screen_y * line_bytes + x * FRAMEBUFFER_BPP;

        let mut buf = [0u8; BUF_SIZE];

        let row_pixels = pixels[row];
        for col in 0..glyph.width() {
            if x + col >= FRAMEBUFFER_WIDTH {
                break;
            }

            let intensity = row_pixels[col];
            if intensity > 0 {
                let buf_off = col * FRAMEBUFFER_BPP;
                buf[buf_off + 0] = b;
                buf[buf_off + 1] = g;
                buf[buf_off + 2] = r;
                buf[buf_off + 3] = a;
            }
        }

        lseek(fd, row_offset, 0).unwrap();
        let bytes_to_write = glyph.width() * FRAMEBUFFER_BPP;
        write(fd, &buf[..bytes_to_write]).unwrap();
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
}
