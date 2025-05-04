use core::{arch::asm, ptr::addr_of};

use alloc::format;
use limine::{request::SmpRequest, smp::Cpu};
use log::info;
use x86_64::{
    instructions::interrupts::enable,
    structures::{
        gdt::GlobalDescriptorTable, idt::InterruptDescriptorTable, tss::TaskStateSegment,
    },
};

use crate::{
    arch::x86::{
        gdt::{self, IA32_GS_BASE, IA32_KERNEL_GS_BASE},
        idt, interrupts,
    },
    hcf,
    sched::task::Task,
    syscall,
};

use super::{gdt::Selectors, sched::SchedulerInfo};

#[used]
#[link_section = ".requests"]
static SMP_REQUEST: SmpRequest = SmpRequest::new();

pub const MAX_CPUS: usize = 4;
pub static mut PCRS: [ProcessorControlRegion; MAX_CPUS] = [
    ProcessorControlRegion::new(),
    ProcessorControlRegion::new(),
    ProcessorControlRegion::new(),
    ProcessorControlRegion::new(),
];

#[repr(C)]
#[derive(Debug)]
pub struct ProcessorControlRegion {
    pub id: u64,
    pub lapic_id: u64,
    pub user_rsp: u64,
    pub kernel_rsp: u64,
    pub tss: TaskStateSegment,
    pub gdt: GlobalDescriptorTable,
    pub idt: InterruptDescriptorTable,
    pub sched: SchedulerInfo,
    pub selectors: Option<Selectors>,
    idle_task: Option<Task>,
}

impl ProcessorControlRegion {
    pub const fn new() -> Self {
        Self {
            id: 0,
            lapic_id: 0,
            user_rsp: 0,
            kernel_rsp: 0,
            tss: TaskStateSegment::new(),
            gdt: GlobalDescriptorTable::new(),
            idt: InterruptDescriptorTable::new(),
            sched: SchedulerInfo::new(),
            selectors: None,
            idle_task: None,
        }
    }

    pub fn idle_task(&mut self) -> &Task {
        if self.idle_task.is_none() {
            self.idle_task = Some(Task::idle());
        }

        self.idle_task.as_ref().unwrap()
    }
}

pub fn init() {
    info!("Starting CPU initialization");

    if let Some(res) = SMP_REQUEST.get_response() {
        let bsp_id = res.bsp_lapic_id();
        info!("BSP LAPIC ID: {}", bsp_id);

        let mut bsp_cpu_info = None;

        for cpu in res.cpus() {
            if cpu.lapic_id == bsp_id {
                bsp_cpu_info = Some(cpu);
                info!(
                    "Found BSP: CPU core {} with LAPIC ID {}",
                    cpu.id, cpu.lapic_id
                );
            } else {
                info!(
                    "Starting AP: CPU core {} with LAPIC ID {}",
                    cpu.id, cpu.lapic_id
                );
                cpu.goto_address.write(cpu_entry);
            }
        }

        if let Some(bsp) = bsp_cpu_info {
            info!("Initializing BSP (CPU core {})", bsp.id);
            init_cpu(bsp);
        } else {
            panic!("Could not find BSP information!");
        }
    } else {
        panic!("SMP information not available!");
    }
}

extern "C" fn cpu_entry(cpu: &Cpu) -> ! {
    init_cpu(cpu);

    hcf();
}

fn init_cpu(cpu: &Cpu) {
    info!("Initializing CPU core {}", cpu.id);
    let pcr = get_pcr_mut(cpu.id as u64);
    pcr.id = cpu.id as u64;
    pcr.lapic_id = cpu.lapic_id as u64;

    let pcr_addr = unsafe { addr_of!(PCRS[cpu.id as usize]) as u64 };
    let pcr_addr = pcr_addr as u64;
    let low = pcr_addr & 0xFFFF_FFFF;
    let high = pcr_addr >> 32;

    unsafe {
        asm!(
            "wrmsr",
            in("rcx") IA32_KERNEL_GS_BASE,
            in("rax") low,
            in("rdx") high,
        );
        asm!(
            "wrmsr",
            in("rcx") IA32_GS_BASE,
            in("rax") low,
            in("rdx") high,
        );
    }

    gdt::init(cpu.id);
    idt::init(cpu.id);
    interrupts::init();
    syscall::init(cpu.id);

    info!("Initialized cpu {}", cpu.id);
    enable();
}

fn get_pcr(cpu_id: u64) -> &'static ProcessorControlRegion {
    // this is safe because we are in the kernel and we know the cpu_id is valid
    // plus each cpu has its own PCR and only it can change it
    unsafe {
        PCRS.get(cpu_id as usize)
            .expect(format!("Failed to get PCR for CPU {}", cpu_id).as_str())
    }
}

fn get_pcr_mut(cpu_id: u64) -> &'static mut ProcessorControlRegion {
    // this is safe because we are in the kernel and we know the cpu_id is valid
    // plus each cpu has its own PCR and only it can change it
    unsafe {
        PCRS.get_mut(cpu_id as usize)
            .expect(format!("Failed to get PCR for CPU {}", cpu_id).as_str())
    }
}

pub fn current_pcr() -> &'static ProcessorControlRegion {
    let low: u32;
    let high: u32;
    unsafe {
        asm!(
            "rdmsr",
            in("rcx") IA32_GS_BASE,
            out("rax") low,
            out("rdx") high,
        );
    }
    let pcr_addr = ((high as u64) << 32) | (low as u64);
    let pcr = pcr_addr as *const ProcessorControlRegion;

    unsafe { &*pcr }
}

pub fn current_pcr_mut() -> &'static mut ProcessorControlRegion {
    let low: u32;
    let high: u32;
    unsafe {
        asm!(
            "rdmsr",
            in("rcx") IA32_GS_BASE,
            out("rax") low,
            out("rdx") high,
        );
    }
    let pcr_addr = ((high as u64) << 32) | (low as u64);
    let pcr = pcr_addr as *mut ProcessorControlRegion;

    unsafe { &mut *pcr }
}
