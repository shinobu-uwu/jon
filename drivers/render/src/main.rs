#![no_std]
#![no_main]

use core::fmt::{Arguments, Write};
use font_constants::BACKUP_CHAR;
use heapless::String;
use jon_common::{
    daemon::get_daemon_pid,
    ipc::{Message, MessageType},
    syscall::fs::{open, read, write},
};
use noto_sans_mono_bitmap::{FontWeight, RasterHeight, RasterizedChar, get_raster};
use spinning_top::Spinlock;

static FRAMEBUFFER_FD: Spinlock<usize> = Spinlock::new(0);
static RANDOM_READ_FD: Spinlock<usize> = Spinlock::new(0);
static RANDOM_WRITE_FD: Spinlock<usize> = Spinlock::new(0);
static SERIAL_FD: Spinlock<usize> = Spinlock::new(0);
const COUNT_MAX: usize = 1000;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    init();

    let mut count: usize = 0;

    loop {
        if count < COUNT_MAX {
            count += 1;
            continue;
        }

        count = 0;
        let render_char = get_char_raster(random_char());
        log(format_args!("Random char: "));
    }
}

fn init() {
    *SERIAL_FD.lock() = open("serial:", 0x0).unwrap();
    log(format_args!("Initializing render"));
    let result = open("vga:0", 0x3);
    log(format_args!("Opened framebuffer: {:#?}", result));
    *FRAMEBUFFER_FD.lock() = result.unwrap();
    let random_pid = get_daemon_pid("random");

    match random_pid {
        Some(pid) => {
            log(format_args!("Random PID: {:#?}", pid));
            let mut read_pipe_path = String::<32>::new();
            write!(read_pipe_path, "pipe:{}/read", pid).unwrap();
            let mut write_pipe_path = String::<32>::new();
            write!(write_pipe_path, "pipe:{}/write", pid).unwrap();
            *RANDOM_READ_FD.lock() = open(&read_pipe_path, 0x1).unwrap();
            *RANDOM_WRITE_FD.lock() = open(&write_pipe_path, 0x2).unwrap();
        }
        None => {
            log(format_args!("Random PID not found"));
        }
    }
}

fn log(args: Arguments) {
    let mut message = String::<128>::new();
    write!(message, "{}", args).unwrap();
    write(*SERIAL_FD.lock(), message.as_bytes()).unwrap();
}

fn random_number() -> usize {
    write(
        *RANDOM_READ_FD.lock(),
        Message::new(MessageType::Read, [0; 16]).to_bytes(),
    )
    .unwrap();

    let mut buf = [0u8; 8];
    let mut result = read(*RANDOM_WRITE_FD.lock(), &mut buf);

    while let Err(err) = result {
        if err == 11 {
            // EAGAIN: no data yet, try again
            result = read(*RANDOM_WRITE_FD.lock(), &mut buf);
            continue;
        } else {
            log(format_args!("Error reading random: {:#?}", err));
            break;
        }
    }

    let bytes_read = result.unwrap();
    usize::from_ne_bytes(buf[..bytes_read].try_into().unwrap())
}

fn random_char() -> char {
    let n = random_number();

    match n % 26 {
        0 => 'A',
        1 => 'B',
        2 => 'C',
        3 => 'D',
        4 => 'E',
        5 => 'F',
        6 => 'G',
        7 => 'H',
        8 => 'I',
        9 => 'J',
        10 => 'K',
        11 => 'L',
        12 => 'M',
        13 => 'N',
        14 => 'O',
        15 => 'P',
        16 => 'Q',
        17 => 'R',
        18 => 'S',
        19 => 'T',
        20 => 'U',
        21 => 'V',
        22 => 'W',
        23 => 'X',
        24 => 'Y',
        25 => 'Z',
        _ => unreachable!(),
    }
}

mod font_constants {
    use super::*;

    pub const CHAR_RASTER_HEIGHT: RasterHeight = RasterHeight::Size16;

    pub const BACKUP_CHAR: char = '�';

    pub const FONT_WEIGHT: FontWeight = FontWeight::Regular;
}

/// Returns the raster of the given char or the raster of [`font_constants::BACKUP_CHAR`].
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

fn write_rendered_char_at(
    framebuffer: &mut [u8],
    rendered_char: RasterizedChar,
    cursor_x: usize,
    cursor_y: usize,
    fb_width: usize,
) {
    for (dy, row) in rendered_char.raster().iter().enumerate() {
        for (dx, byte) in row.iter().enumerate() {
            write_pixel(framebuffer, cursor_x + dx, cursor_y + dy, *byte, fb_width);
        }
    }
}

fn write_pixel(
    framebuffer: &mut [u8],
    x: usize,
    y: usize,
    intensity: u8,
    fb_width: usize, // real framebuffer width in pixels
) {
    let pixel_offset = y * fb_width + x;
    let color = [intensity, intensity, intensity / 2, 0xFF]; // RGBA
    let bytes_per_pixel = 4;
    let byte_offset = pixel_offset * bytes_per_pixel;
    if byte_offset + bytes_per_pixel <= framebuffer.len() {
        framebuffer[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
    }
}
