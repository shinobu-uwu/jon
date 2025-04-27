use core::ptr::addr_of;

use limine::{request::SmpRequest, smp::Cpu};
use log::{error, info};
use x86_64::{
    instructions::tables::load_tss,
    registers::{
        control::{Efer, EferFlags},
        model_specific::{LStar, SFMask, Star},
        rflags::RFlags,
        segmentation::{Segment, CS, SS},
    },
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable},
        idt::{InterruptDescriptorTable, InterruptStackFrame},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

use crate::{
    arch::x86::{
        gdt::{ProcessorControlRegion, DOUBLE_FAULT_IST_INDEX, IA32_GS_BASE, IA32_KERNEL_GS_BASE},
        restore,
    },
    sched::task::Task,
    syscall::syscall_instruction,
};

#[used]
#[link_section = ".requests"]
static SMP_REQUEST: SmpRequest = SmpRequest::new();
static mut TSS: TaskStateSegment = TaskStateSegment::new();
static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable::new();
static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();
static mut PCR: ProcessorControlRegion = ProcessorControlRegion {
    user_rsp: 0,
    kernel_rsp: 0,
};

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

    let task_manager = Task::new(
        "task_manager",
        include_bytes!(
            "../../../drivers/task_manager/target/x86_64-unknown-none/release/task_manager"
        ),
    );
    info!("{:#x?}", task_manager.context);
    info!("CPU setup complete, switching to task manager");
    unsafe { restore(&task_manager.context) }
}

unsafe fn init_cpu() {
    let kernel_code_selector = GDT.append(Descriptor::kernel_code_segment());
    let kernel_data_selector = GDT.append(Descriptor::kernel_data_segment());
    let user_data_selector = GDT.append(Descriptor::user_data_segment());
    let user_code_selector = GDT.append(Descriptor::user_code_segment());

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
    let pcr_addr = addr_of!(PCR) as u64;

    core::arch::asm!(
        "wrmsr",
        in("rcx") IA32_KERNEL_GS_BASE,
        in("rax") pcr_addr,
        in("rdx") pcr_addr >> 32,
    );
    core::arch::asm!(
        "wrmsr",
        in("rcx") IA32_GS_BASE,
        in("rax") pcr_addr,
        in("rdx") pcr_addr >> 32,
    );
    IDT.load();

    unsafe {
        Efer::update(|efer| {
            efer.insert(EferFlags::SYSTEM_CALL_EXTENSIONS);
        });
    }

    let (kernel_cs, kernel_ss, user_cs, user_ss) = (
        kernel_code_selector,
        kernel_data_selector,
        user_code_selector,
        user_data_selector,
    );

    match Star::write(user_cs, user_ss, kernel_cs, kernel_ss) {
        Ok(_) => {
            info!("STAR MSR set successfully");
        }
        Err(e) => {
            panic!("Error setting STAR: {}", e)
        }
    }

    LStar::write(VirtAddr::new(syscall_instruction as u64));
    SFMask::write(RFlags::INTERRUPT_FLAG);
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
