#![no_std]
#![no_main]

use jon_common::syscall::fs::{open, write};
use spinning_top::Spinlock;
use ui::{FONT_SIZE, Framebuffer, screen::Screen};
use writer::FramebufferWriter;

mod allocator;
mod proc;
mod ui;
mod writer;

extern crate alloc;

static SERIAL_FD: Spinlock<usize> = Spinlock::new(0);
pub const Y_OFFSET: usize = FONT_SIZE.val() * 2 + 8;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    init();
    let fb_fd = open("vga:0", 0x0).unwrap();
    let fb = Framebuffer::default();
    let writer = FramebufferWriter::new(fb_fd, fb);
    let mut screen = Screen::new(writer);

    loop {
        screen.draw();
    }
}

fn init() {
    allocator::init();
    *SERIAL_FD.lock() = open("serial:", 0x0).unwrap();
}

fn log(message: &str) {
    write(*SERIAL_FD.lock(), message.as_bytes()).unwrap();
}
