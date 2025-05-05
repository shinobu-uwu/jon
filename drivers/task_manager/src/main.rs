#![no_std]
#![no_main]

use core::ffi::CStr;

use alloc::{format, vec::Vec};
use jon_common::syscall::{
    fs::{open, read, write},
    task::kill,
};
use pc_keyboard::{DecodedKey, HandleControl, KeyCode, Keyboard, ScancodeSet2, layouts};
use proc::{Proc, State};
use ui::{Color, FONT_SIZE, Framebuffer, FramebufferWriter};

mod allocator;
mod proc;
mod ui;

extern crate alloc;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    allocator::init();
    let serial_fd = open("serial:", 0x0).unwrap();
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
    let mut buf = [0u8; 128 * core::mem::size_of::<Proc>()];
    let mut selected_proc: usize = 0;

    loop {
        writer.clear();

        let bytes_read = read(proc_fd, &mut buf).unwrap();
        let procs_buf = &buf[..bytes_read];
        let procs: Vec<Proc> = procs_buf
            .windows(core::mem::size_of::<Proc>())
            .step_by(core::mem::size_of::<Proc>())
            .map(|bytes| Proc::from_bytes(bytes))
            .collect();
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

            let color = match proc.state {
                State::Running => Color::Green,
                State::Blocked => Color::Red,
                State::Waiting => Color::Yellow,
                State::Stopped => Color::Cyan,
            };
            let state_label = match proc.state {
                State::Running => "Rodando",
                State::Blocked => "Bloqueado",
                State::Waiting => "Esperando",
                State::Stopped => "Parada",
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
                            DecodedKey::Unicode(k) => {
                                if k == 'k' {
                                    write(serial_fd, b"Killing task!!").unwrap();
                                    match kill(procs[selected_proc].pid) {
                                        Ok(f) => {
                                            let found = f != 0;
                                            write(
                                                serial_fd,
                                                format!("Task killed: {}", found).as_bytes(),
                                            )
                                            .unwrap();
                                        }
                                        Err(e) => {
                                            write(
                                                serial_fd,
                                                format!("Error killing task: {}", e).as_bytes(),
                                            )
                                            .unwrap();
                                        }
                                    }
                                }
                            }
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
                                        serial_fd,
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
