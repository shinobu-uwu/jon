#![no_std]
#![no_main]

mod fb;

use core::fmt::{Arguments, Write};
use fb::{FramebufferInfo, FramebufferWriter};
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
static mut LOCAL_FB: [u8; 64 * 64 * 4] = [0; 64 * 64 * 4];
const COUNT_MAX: usize = 10000000;

#[unsafe(no_mangle)]
#[allow(static_mut_refs)]
pub extern "C" fn _start() -> ! {
    init();

    let mut count: usize = 0;
    let mut writer =
        unsafe { FramebufferWriter::new(&mut LOCAL_FB, FramebufferInfo::new(64, 64, 64, 4)) };

    loop {
        if count < COUNT_MAX {
            count += 1;
            continue;
        }

        count = 0;
        let char = random_char();
        log(format_args!("Printing char {}", char));
        writer.write_char(char).unwrap();
        unsafe {
            match write(*FRAMEBUFFER_FD.lock(), &LOCAL_FB) {
                Ok(_) => log(format_args!("Wrote framebuffer")),
                Err(err) => log(format_args!("Error writing framebuffer: {:#?}", err)),
            }
        }
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
