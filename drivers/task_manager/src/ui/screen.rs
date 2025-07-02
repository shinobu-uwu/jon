use core::ffi::CStr;

use alloc::{format, vec::Vec};
use jon_common::syscall::{
    fs::{open, read},
    task::spawn,
};
use pc_keyboard::{DecodedKey, HandleControl, KeyCode, Keyboard, ScancodeSet2, layouts};

use crate::{
    Y_OFFSET, log,
    proc::{Proc, State, kill_proc, list_procs},
    writer::FramebufferWriter,
};

use super::{Color, FONT_SIZE};

const PADDING: usize = 8;
const NEW_PROCS: [&str; 2] = ["random", "random-echo"];

pub struct Screen {
    pub screen_state: ScreenState,
    writer: FramebufferWriter,
    proc_fd: usize,
    keyboard_fd: usize,
    keyboard: Keyboard<layouts::Us104Key, ScancodeSet2>,
    selected_proc: usize,
    procs: Vec<Proc>,
}

pub enum ScreenState {
    Selection,
    Spawn,
}

impl Screen {
    pub fn new(writer: FramebufferWriter) -> Self {
        let proc_fd = open("proc:", 0x0).unwrap();
        let keyboard_fd = open("ps2:", 0x0).unwrap();
        let keyboard = Keyboard::new(
            ScancodeSet2::new(),
            layouts::Us104Key,
            HandleControl::Ignore,
        );
        Self {
            screen_state: ScreenState::Selection,
            writer,
            proc_fd,
            keyboard_fd,
            keyboard,
            selected_proc: 0,
            procs: Vec::new(),
        }
    }

    #[inline(always)]
    fn legend_offset(&self) -> usize {
        self.writer.height() - FONT_SIZE.val() - PADDING
    }

    pub fn draw(&mut self) {
        self.writer.clear();

        match self.screen_state {
            ScreenState::Selection => self.draw_selection(),
            ScreenState::Spawn => self.draw_spawn(),
        }

        self.read_keyboard();
        self.writer.flush();
    }

    fn draw_header(&mut self) {
        self.writer.write_text(
            0,
            0,
            "Task Manager - Use as setas para navegar",
            Color::White,
        );
    }

    fn draw_selection(&mut self) {
        self.procs = list_procs(self.proc_fd);
        self.draw_header();
        self.writer.write_text(
            0,
            FONT_SIZE.val() + PADDING,
            "PID NOME             ESTADO",
            Color::White,
        );
        self.writer.write_text(
            0,
            self.legend_offset(),
            "K - Matar | N - Novo",
            Color::White,
        );

        for (i, proc) in self.procs.iter().enumerate() {
            let row_y = Y_OFFSET + i * (FONT_SIZE.val() + PADDING);

            if i == self.selected_proc {
                self.writer.draw_line(row_y, FONT_SIZE.val(), Color::Blue);
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
            self.writer.write_text(0, row_y, &text, color);
        }
    }

    fn draw_spawn(&mut self) {
        self.draw_header();
        let legend_offset = self.legend_offset();
        self.writer.write_text(
            0,
            legend_offset,
            "S - Criar processo | Q - Sair",
            Color::White,
        );

        for (i, proc) in NEW_PROCS.iter().enumerate() {
            let row_y = Y_OFFSET + i * (FONT_SIZE.val() + PADDING);

            if i == self.selected_proc {
                self.writer.draw_line(row_y, FONT_SIZE.val(), Color::Blue);
            }

            let text = format!("{}", proc);
            self.writer.write_text(0, row_y, &text, Color::Green);
        }
    }

    fn read_keyboard(&mut self) {
        let mut buf = [0u8; 3];
        match read(self.keyboard_fd, &mut buf) {
            Ok(_) => {
                if let Ok(Some(key_event)) = self.keyboard.add_byte(buf[0]) {
                    if let Some(key) = self.keyboard.process_keyevent(key_event) {
                        self.handle_key(key);
                    }
                }
            }
            Err(_) => return,
        }
    }

    fn handle_key(&mut self, key: DecodedKey) {
        match self.screen_state {
            ScreenState::Selection => match key {
                DecodedKey::RawKey(KeyCode::ArrowUp) => {
                    if self.selected_proc > 0 {
                        self.selected_proc -= 1;
                    }
                }
                DecodedKey::RawKey(KeyCode::ArrowDown) => {
                    if self.selected_proc < self.procs.len() - 1 {
                        self.selected_proc += 1;
                    }
                }
                DecodedKey::Unicode('k') => {
                    let proc = &self.procs[self.selected_proc];
                    kill_proc(proc);
                }
                DecodedKey::Unicode('n') => {
                    self.writer.force_clear();
                    self.selected_proc = 0;
                    self.screen_state = ScreenState::Spawn;
                }
                _ => {}
            },
            ScreenState::Spawn => match key {
                DecodedKey::RawKey(KeyCode::ArrowUp) => {
                    if self.selected_proc > 0 {
                        self.selected_proc -= 1;
                    }
                }
                DecodedKey::RawKey(KeyCode::ArrowDown) => {
                    if self.selected_proc < self.procs.len() - 1 {
                        self.selected_proc += 1;
                    }
                }
                DecodedKey::Unicode('q') => {
                    self.writer.force_clear();
                    self.selected_proc = 0;
                    self.screen_state = ScreenState::Selection;
                }
                DecodedKey::Unicode('s') => match spawn(self.selected_proc + 2) {
                    Ok(_) => {}
                    Err(e) => log(&format!(
                        "Falha ao criar processo {}: {}",
                        NEW_PROCS[self.selected_proc], e
                    )),
                },
                _ => {}
            },
        }
    }
}
