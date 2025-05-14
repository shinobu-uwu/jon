#![no_std]
#![no_main]

use core::{ffi::CStr, mem::size_of};

use alloc::{format, vec::Vec};
use jon_common::{
    ipc::Message,
    syscall::{
        fs::{open, read, write},
        task::kill,
    },
};
use pc_keyboard::{DecodedKey, HandleControl, KeyCode, Keyboard, ScancodeSet2, layouts};
use proc::{Proc, State};
use spinning_top::Spinlock;
use ui::{Color, FONT_SIZE, Framebuffer, FramebufferWriter};

mod allocator;
mod proc;
mod ui;

extern crate alloc;

static SERIAL_FD: Spinlock<usize> = Spinlock::new(0);

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    init();
    let fb_fd = open("vga:0", 0x0).unwrap();
    let keyboard_fd = open("ps2:", 0x0).unwrap();
    let mut kb_buf = [0u8; 3];
    let mut keyboard = Keyboard::new(
        ScancodeSet2::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );
    let proc_fd = open("proc:", 0x0).unwrap();
    let fb = Framebuffer::default();
    let mut writer = FramebufferWriter::new(fb_fd, fb);
    let mut selected_proc: usize = 0;

    loop {
        writer.clear();
        let procs = list_procs(proc_fd);

        writer.write_text(
            0,
            0,
            "Task Manager - Use as setas para navegar",
            Color::White,
        );
        writer.write_text(
            0,
            FONT_SIZE.val() + 8,
            "PID NOME             ESTADO",
            Color::White,
        );
        let y_offset = FONT_SIZE.val() * 2 + 8;

        for (i, proc) in procs.iter().enumerate() {
            let row_y = y_offset + i * (FONT_SIZE.val() + 8);

            if i == selected_proc {
                writer.draw_line(row_y, FONT_SIZE.val(), Color::Blue);
            }

            let (color, state_label) = match proc.state {
                State::Running => (Color::Green, "Rodando"),
                State::Blocked => (Color::Cyan, "Bloqueado"),
                State::Waiting => (Color::Yellow, "Esperando"),
                State::Stopped => (Color::Red, "Parada"),
            };

            let name = CStr::from_bytes_until_nul(&proc.name)
                .unwrap()
                .to_str()
                .unwrap();
            let text = format!("{:>3} {:<16} {:<10}", proc.pid, name, state_label);
            writer.write_text(0, row_y, &text, color);
        }

        let legend_offset = writer.height() - FONT_SIZE.val() - 8;
        writer.write_text(
            0,
            legend_offset,
            "K - Kill | R - Restart | S - Sleep",
            Color::White,
        );

        writer.flush();

        match read(keyboard_fd, &mut kb_buf) {
            Ok(_) => {
                if let Ok(Some(key_event)) = keyboard.add_byte(kb_buf[0]) {
                    if let Some(key) = keyboard.process_keyevent(key_event) {
                        match key {
                            DecodedKey::Unicode(k) => match k {
                                'k' => {
                                    let proc = &procs[selected_proc];
                                    kill_proc(proc);
                                }
                                'r' => {
                                    let proc = &procs[selected_proc];
                                    restart_proc(proc);
                                }
                                _ => {}
                            },
                            DecodedKey::RawKey(k) => match k {
                                KeyCode::ArrowUp => {
                                    if selected_proc > 0 {
                                        selected_proc -= 1;
                                    }
                                }
                                KeyCode::ArrowDown => {
                                    if selected_proc < procs.len() - 1 {
                                        selected_proc += 1;
                                    }
                                }
                                KeyCode::Return => {
                                    let pid = procs[selected_proc].pid;
                                    write(
                                        *SERIAL_FD.lock(),
                                        format!("Restarting task with PID: {}", pid).as_bytes(),
                                    )
                                    .unwrap();
                                }
                                _ => {}
                            },
                        }
                    }
                }
            }
            Err(_) => continue,
        }
    }
}

fn init() {
    allocator::init();
    *SERIAL_FD.lock() = open("serial:", 0x0).unwrap();
}

fn log(message: &str) {
    write(*SERIAL_FD.lock(), message.as_bytes()).unwrap();
}

fn list_procs(proc_fd: usize) -> Vec<Proc> {
    let mut buf = [0u8; 128 * size_of::<Proc>()];
    let bytes_read = read(proc_fd, &mut buf).unwrap();
    let procs_buf = &buf[..bytes_read];

    procs_buf
        .windows(size_of::<Proc>())
        .step_by(size_of::<Proc>())
        .map(|bytes| Proc::from_bytes(bytes))
        .collect()
}

fn kill_proc(proc: &Proc) {
    if proc.state != State::Running && proc.state != State::Waiting {
        write(*SERIAL_FD.lock(), b"Task not running, cannot kill").unwrap();
        return;
    }

    log("Attempting to kill task...");

    match kill(proc.pid) {
        Ok(f) => {
            let found = f != 0;
            write(
                *SERIAL_FD.lock(),
                format!("Task killed: {}", found).as_bytes(),
            )
            .unwrap();
        }
        Err(e) => {
            write(
                *SERIAL_FD.lock(),
                format!("Error killing task: {}", e).as_bytes(),
            )
            .unwrap();
        }
    }

    log("Sending kill message...");
    let fd = open("pipe:1/read", 0x2).unwrap();
    log("Writing to pipe...");
    write(
        fd,
        Message::new(jon_common::ipc::MessageType::Delete, proc.name).to_bytes(),
    )
    .unwrap();
    log("Message sent.");
}

fn restart_proc(proc: &Proc) {}
