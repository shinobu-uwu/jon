use core::{
    fmt::{self, Write},
    ptr,
};
use font_constants::BACKUP_CHAR;
use limine::{framebuffer::Framebuffer, request::FramebufferRequest};
use noto_sans_mono_bitmap::{
    get_raster, get_raster_width, FontWeight, RasterHeight, RasterizedChar,
};
use spinning_top::Spinlock;

const LINE_SPACING: usize = 4;
const LETTER_SPACING: usize = 2;
const BORDER_PADDING: usize = 4;

#[used]
#[link_section = ".requests"]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();
pub static WRITER: Spinlock<Option<FrameBufferWriter>> = Spinlock::new(None);

#[macro_export]
macro_rules! tty_print {
    ($($arg:tt)*) => ($crate::output::tty::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! tty_println {
    () => ($crate::tty_print!("\n"));
    ($($arg:tt)*) => ($crate::tty_print!("{}\n", format_args!($($arg)*)));
}

pub struct FrameBufferInfo {
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub bytes_per_pixel: usize,
    pixel_format: PixelFormat,
}

#[derive(Debug, Clone, Copy)]
enum PixelFormat {
    Rgb,
    Bgr,
    U8,
}

mod font_constants {
    use super::*;
    pub const CHAR_RASTER_HEIGHT: RasterHeight = RasterHeight::Size16;
    pub const CHAR_RASTER_WIDTH: usize = get_raster_width(FontWeight::Regular, CHAR_RASTER_HEIGHT);
    pub const BACKUP_CHAR: char = '�';
    pub const FONT_WEIGHT: FontWeight = FontWeight::Regular;
}

pub struct FrameBufferWriter {
    framebuffer: &'static mut [u8],
    info: FrameBufferInfo,
    x_pos: usize,
    y_pos: usize,
}

impl FrameBufferWriter {
    pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        let mut writer = Self {
            framebuffer,
            info,
            x_pos: 0,
            y_pos: 0,
        };
        writer.clear();
        writer
    }

    pub fn clear(&mut self) {
        self.x_pos = BORDER_PADDING;
        self.y_pos = BORDER_PADDING;
        self.framebuffer.fill(255);
    }

    fn newline(&mut self) {
        self.y_pos += font_constants::CHAR_RASTER_HEIGHT.val() + LINE_SPACING;
        self.carriage_return()
    }

    fn carriage_return(&mut self) {
        self.x_pos = BORDER_PADDING;
    }

    fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            '\r' => self.carriage_return(),
            c => {
                let new_xpos = self.x_pos + font_constants::CHAR_RASTER_WIDTH;
                if new_xpos >= self.info.width {
                    self.newline();
                }

                let new_ypos =
                    self.y_pos + font_constants::CHAR_RASTER_HEIGHT.val() + BORDER_PADDING;
                if new_ypos >= self.info.height {
                    self.clear();
                }

                self.write_rendered_char(get_char_raster(c));
            }
        }
    }

    fn write_rendered_char(&mut self, rendered_char: RasterizedChar) {
        for (y, row) in rendered_char.raster().iter().enumerate() {
            for (x, byte) in row.iter().enumerate() {
                self.write_pixel(self.x_pos + x, self.y_pos + y, *byte);
            }
        }
        self.x_pos += rendered_char.width() + LETTER_SPACING;
    }

    fn write_pixel(&mut self, x: usize, y: usize, intensity: u8) {
        let pixel_offset = y * self.info.stride + x;
        let color = match self.info.pixel_format {
            PixelFormat::Rgb => [intensity, intensity, intensity, 0], // White text (R=G=B)
            PixelFormat::Bgr => [intensity, intensity, intensity, 0], // White text (B=G=R)
            PixelFormat::U8 => [if intensity > 200 { 0xff } else { 0 }, 0, 0, 0], // Binary black/white
        };

        let bytes_per_pixel = self.info.bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;
        self.framebuffer[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
        let _ = unsafe { ptr::read_volatile(&self.framebuffer[byte_offset]) };
    }
}

impl fmt::Write for FrameBufferWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}

fn framebuffer_info_from_limine(framebuffer: &Framebuffer) -> FrameBufferInfo {
    FrameBufferInfo {
        width: framebuffer.width() as usize,
        height: framebuffer.height() as usize,
        stride: framebuffer.pitch() as usize,
        bytes_per_pixel: framebuffer.bpp() as usize / 8,
        pixel_format: PixelFormat::Rgb,
    }
}

pub fn init_tty() {
    let response = FRAMEBUFFER_REQUEST.get_response().unwrap();
    if let Some(framebuffer) = response.framebuffers().next() {
        let info = framebuffer_info_from_limine(&framebuffer);
        *WRITER.lock() = Some(FrameBufferWriter::new(
            unsafe {
                core::slice::from_raw_parts_mut(
                    framebuffer.addr() as *mut u8,
                    framebuffer.height() as usize * framebuffer.pitch() as usize,
                )
            },
            info,
        ));
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    WRITER.lock().as_mut().unwrap().write_fmt(args).unwrap();
}

fn get_char_raster(c: char) -> RasterizedChar {
    fn get(c: char) -> Option<RasterizedChar> {
        get_raster(
            c,
            font_constants::FONT_WEIGHT,
            font_constants::CHAR_RASTER_HEIGHT,
        )
    }
    get(c).unwrap_or_else(|| get(BACKUP_CHAR).expect("Should get raster of backup char."))
}
