mod ui;

use alloc::format;
use core::{ptr::addr_of, u8};
use pc_keyboard::{layouts, DecodedKey, HandleControl, KeyCode, Keyboard, ScancodeSet2};

use limine::{request::SmpRequest, smp::Cpu};
use log::{error, info};
use ui::{get_key, Color, FramebufferWriter, FONT_SIZE};
use x86_64::{
    instructions::tables::load_tss,
    registers::segmentation::{Segment, CS, SS},
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable},
        idt::{InterruptDescriptorTable, InterruptStackFrame},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

use crate::{
    arch::x86::gdt::DOUBLE_FAULT_IST_INDEX,
    sched::{scheduler::get_tasks, task::State},
    scheme::vga::{init_fbs, FRAMEBUFFERS},
};

#[used]
#[link_section = ".requests"]
static SMP_REQUEST: SmpRequest = SmpRequest::new();
static mut TSS: TaskStateSegment = TaskStateSegment::new();
static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable::new();
static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub fn start_task_manager() {
    info!("Starting task manager");

    if let Some(res) = SMP_REQUEST.get_response() {
        if res.cpus().len() < 2 {
            error!("Not enough CPU cores available for task manager");
            return;
        }

        let cpu = &res.cpus()[1];
        info!("Starting task manager on CPU core {}", cpu.id);
        cpu.goto_address.write(task_manager_entry);
    }
}

extern "C" fn task_manager_entry(_cpu: &Cpu) -> ! {
    info!("Task manager core initialized");
    unsafe {
        init_cpu();
    }

    init_fbs();
    let mut framebuffers = FRAMEBUFFERS.write();
    let framebuffer = framebuffers.get_mut(0).unwrap();
    let mut writer = FramebufferWriter::new(framebuffer);
    let mut selected_task = 0usize;
    let mut keyboard = Keyboard::new(
        ScancodeSet2::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    loop {
        writer.clear();
        let tasks = get_tasks();

        if let Some(key) = get_key(&mut keyboard) {
            if let DecodedKey::RawKey(k) = key {
                if k == KeyCode::ArrowUp {
                    selected_task = selected_task.wrapping_sub(1).min(tasks.len() - 1);
                } else if k == KeyCode::ArrowDown {
                    selected_task = selected_task.wrapping_add(1).min(tasks.len() - 1);
                }
            };
        }

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

        for (i, task) in tasks.iter().enumerate() {
            let row_y = y_offset + i * (FONT_SIZE.val() + 8);

            if i == selected_task {
                writer.draw_line(row_y, FONT_SIZE.val(), Color::Blue);
            }

            let color = match task.state {
                State::Running => Color::Green,
                State::Blocked => Color::Red,
                State::Waiting => Color::Yellow,
                State::Zombie => Color::Cyan,
            };
            let state_label = match task.state {
                State::Running => "Rodando",
                State::Blocked => "Bloqueado",
                State::Waiting => "Esperando",
                State::Zombie => "Zumbi",
            };

            let text = format!(
                "{:>3} {:<16} {:<10}",
                task.pid.as_64(),
                task.name,
                state_label
            );
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
    }
}

unsafe fn init_cpu() {
    let kernel_code_selector = GDT.append(Descriptor::kernel_code_segment());
    let kernel_data_selector = GDT.append(Descriptor::kernel_data_segment());

    const STACK_SIZE: usize = 4096 * 5;
    static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
    let stack_start = VirtAddr::from_ptr(addr_of!(STACK));
    TSS.privilege_stack_table[0] = stack_start + STACK_SIZE as u64;
    TSS.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
        const STACK_SIZE: usize = 4096 * 5;
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

        let stack_start = VirtAddr::from_ptr(addr_of!(STACK));
        stack_start + STACK_SIZE as u64
    };

    let tss_selector = GDT.append(Descriptor::tss_segment(&TSS));

    GDT.load();
    IDT.double_fault
        .set_handler_fn(double_fault_handler)
        .set_stack_index(crate::arch::x86::gdt::DOUBLE_FAULT_IST_INDEX);

    CS::set_reg(kernel_code_selector);
    SS::set_reg(kernel_data_selector);

    load_tss(tss_selector);
    IDT.load();
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "EXCEPTION: DOUBLE FAULT\n\
        Stack Frame: {:#?}\n\
        Error Code: {}\n\
        stack_frame,
        error_code,",
        stack_frame, error_code
    );
}
