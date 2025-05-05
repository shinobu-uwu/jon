use crate::arch::end_of_interrupt;
use crate::arch::x86::cpu::{current_pcr, PCRS};
use crate::arch::x86::interrupts::{ERROR_VECTOR, SPURIOUS_VECTOR, TIMER_VECTOR};
use crate::interrupt;
use crate::sched::scheduler::schedule;
use log::{debug, info, warn};
use spinning_top::Spinlock;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};

static LAST_EXCEPTION: Spinlock<Option<ExceptionInfo>> = Spinlock::new(None);

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct ExceptionInfo {
    vector: u32,
    error_code: u32,
    cr2: u64,
}

pub fn init(cpu_id: u32) {
    let pcr = unsafe { PCRS.get_mut(cpu_id as usize).unwrap() };
    debug!("Loading IDT for cpu {}", pcr.id);
    pcr.idt.divide_error.set_handler_fn(divide_error_handler);
    pcr.idt.debug.set_handler_fn(debug_handler);
    pcr.idt.non_maskable_interrupt.set_handler_fn(nmi_handler);
    pcr.idt.breakpoint.set_handler_fn(breakpoint_handler);
    pcr.idt.overflow.set_handler_fn(overflow_handler);
    pcr.idt
        .bound_range_exceeded
        .set_handler_fn(bound_range_handler);
    pcr.idt
        .invalid_opcode
        .set_handler_fn(invalid_opcode_handler);
    pcr.idt
        .device_not_available
        .set_handler_fn(device_not_available_handler);

    unsafe {
        pcr.idt
            .double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(crate::arch::x86::gdt::DOUBLE_FAULT_IST_INDEX);
    }

    pcr.idt.invalid_tss.set_handler_fn(invalid_tss_handler);
    pcr.idt
        .segment_not_present
        .set_handler_fn(segment_not_present_handler);
    pcr.idt
        .stack_segment_fault
        .set_handler_fn(stack_segment_fault_handler);
    pcr.idt
        .general_protection_fault
        .set_handler_fn(general_protection_fault_handler);
    pcr.idt.page_fault.set_handler_fn(page_fault_handler);
    pcr.idt
        .x87_floating_point
        .set_handler_fn(x87_floating_point_handler);
    pcr.idt
        .alignment_check
        .set_handler_fn(alignment_check_handler);
    pcr.idt.machine_check.set_handler_fn(machine_check_handler);
    pcr.idt
        .simd_floating_point
        .set_handler_fn(simd_floating_point_handler);
    pcr.idt
        .virtualization
        .set_handler_fn(virtualization_handler);
    pcr.idt
        .cp_protection_exception
        .set_handler_fn(cp_protection_handler);
    pcr.idt[TIMER_VECTOR as u8].set_handler_fn(timer_interrupt_handler);
    pcr.idt[ERROR_VECTOR as u8].set_handler_fn(error_interrupt_handler);
    pcr.idt[SPURIOUS_VECTOR as u8].set_handler_fn(spurious_interrupt_handler);
    pcr.idt.load();
    debug!("IDT loaded")
}

interrupt!(timer_interrupt_handler, |interrupt_stack| {
    end_of_interrupt();
    schedule(interrupt_stack);
});

// Exception Handlers
extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    set_last_exception(0, 0, 0);
    panic!("EXCEPTION: DIVIDE ERROR\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    set_last_exception(1, 0, 0);
    panic!("EXCEPTION: DEBUG\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn nmi_handler(stack_frame: InterruptStackFrame) {
    set_last_exception(2, 0, 0);
    panic!("EXCEPTION: NON-MASKABLE INTERRUPT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    set_last_exception(3, 0, 0);
    info!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    set_last_exception(4, 0, 0);
    panic!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn bound_range_handler(stack_frame: InterruptStackFrame) {
    set_last_exception(5, 0, 0);
    panic!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    set_last_exception(6, 0, 0);
    panic!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    set_last_exception(7, 0, 0);
    panic!("EXCEPTION: DEVICE NOT AVAILABLE\n{:#?}", stack_frame);
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

extern "x86-interrupt" fn invalid_tss_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    set_last_exception(10, error_code as u32, 0);
    panic!(
        "EXCEPTION: INVALID TSS\nError Code: {}\n{:#?}",
        error_code, stack_frame
    );
}

extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    set_last_exception(11, error_code as u32, 0);
    panic!(
        "EXCEPTION: SEGMENT NOT PRESENT\nError Code: {}\n{:#?}",
        error_code, stack_frame
    );
}

extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    set_last_exception(12, error_code as u32, 0);
    panic!(
        "EXCEPTION: STACK SEGMENT FAULT\nError Code: {}\n{:#?}",
        error_code, stack_frame
    );
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    set_last_exception(13, error_code as u32, 0);
    panic!(
        "EXCEPTION: GENERAL PROTECTION FAULT\nError Code: {}\n{:#?}",
        error_code, stack_frame
    );
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    // if stack_frame.code_segment == GDT.1.user_code_selector {
    //     remove_current_task();
    //     error!("Page fault in user mode, removed running task");
    //     debug!("{:#?}", stack_frame);
    //     return;
    // }

    panic!(
        "EXCEPTION: PAGE FAULT\nAccessed Address: {:?}\nError Code: {:?}\n{:#?}",
        Cr2::read(),
        error_code,
        stack_frame,
    );
}

extern "x86-interrupt" fn x87_floating_point_handler(stack_frame: InterruptStackFrame) {
    set_last_exception(16, 0, 0);
    panic!("EXCEPTION: x87 FLOATING POINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    set_last_exception(17, error_code as u32, 0);
    panic!(
        "EXCEPTION: ALIGNMENT CHECK\nError Code: {}\n{:#?}",
        error_code, stack_frame
    );
}

extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    set_last_exception(18, 0, 0);
    panic!("EXCEPTION: MACHINE CHECK\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) {
    set_last_exception(19, 0, 0);
    panic!("EXCEPTION: SIMD FLOATING POINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn virtualization_handler(stack_frame: InterruptStackFrame) {
    set_last_exception(20, 0, 0);
    panic!("EXCEPTION: VIRTUALIZATION\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn cp_protection_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    set_last_exception(21, error_code as u32, 0);
    panic!(
        "EXCEPTION: CP PROTECTION\nError Code: {}\n{:#?}",
        error_code, stack_frame
    );
}

// LAPIC handlers
extern "x86-interrupt" fn error_interrupt_handler(_frame: InterruptStackFrame) {
    debug!("Handling error");
    info!("Error interrupt");
    unsafe {
        debug!("Notifying end of interrupt");
        end_of_interrupt();
        debug!("Notified end of interrupt");
    }
}

extern "x86-interrupt" fn spurious_interrupt_handler(_frame: InterruptStackFrame) {
    warn!("Handling spurious");
}

fn exception_name(vector: u32) -> &'static str {
    match vector {
        0 => "Divide Error",
        1 => "Debug Exception",
        2 => "NMI Interrupt",
        3 => "Breakpoint",
        4 => "Overflow",
        5 => "BOUND Range Exceeded",
        6 => "Invalid Opcode",
        7 => "Device Not Available",
        8 => "Double Fault",
        9 => "Coprocessor Segment Overrun",
        10 => "Invalid TSS",
        11 => "Segment Not Present",
        12 => "Stack Fault",
        13 => "General Protection",
        14 => "Page Fault",
        16 => "x87 FPU Floating-Point Error",
        17 => "Alignment Check",
        18 => "Machine Check",
        19 => "SIMD Floating-Point Exception",
        20 => "Virtualization Exception",
        21 => "Control Protection Exception",
        _ => "Unknown Exception",
    }
}

impl core::fmt::Display for ExceptionInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{} (vector: {}, error_code: {:#x}, cr2: {:#x})",
            exception_name(self.vector),
            self.vector,
            self.error_code,
            self.cr2
        )
    }
}

#[inline]
fn set_last_exception(vector: u32, error_code: u32, cr2: u64) {
    *LAST_EXCEPTION.lock() = Some(ExceptionInfo {
        vector,
        error_code,
        cr2,
    });
}
