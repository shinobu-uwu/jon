#![no_std]
#![no_main]

mod fb;
mod proc;

use core::{
    ffi::CStr,
    fmt::{Arguments, Write},
};
use fb::{FramebufferInfo, FramebufferWriter};
use heapless::{String, Vec};
use jon_common::{
    daemon::get_daemon_pid,
    ipc::{Message, MessageType},
    syscall::{
        self,
        fs::{open, read, write},
    },
};
use proc::Proc;
use spinning_top::Spinlock;

static FRAMEBUFFER_FD: Spinlock<usize> = Spinlock::new(0);
static RANDOM_READ_FD: Spinlock<usize> = Spinlock::new(0);
static RANDOM_WRITE_FD: Spinlock<usize> = Spinlock::new(0);
static PROC_FD: Spinlock<usize> = Spinlock::new(0);
static SERIAL_FD: Spinlock<usize> = Spinlock::new(0);
const COUNT_MAX: usize = 10000000;

#[allow(static_mut_refs)]
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
        let mut buf = [0u8; 128 * core::mem::size_of::<Proc>()];
        let bytes_read = syscall::fs::read(*PROC_FD.lock(), &mut buf).unwrap();
        let procs_buf = &buf[..bytes_read];
        let procs: Vec<Proc, 128> = procs_buf
            .windows(core::mem::size_of::<Proc>())
            .step_by(core::mem::size_of::<Proc>())
            .map(|bytes| Proc::from_bytes(bytes))
            .collect();

        for proc in procs.iter() {
            let name = CStr::from_bytes_until_nul(&proc.name);
            log(format_args!("Proc: {:?}", name));
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

    *PROC_FD.lock() = open("proc:", 0x0).unwrap();
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
