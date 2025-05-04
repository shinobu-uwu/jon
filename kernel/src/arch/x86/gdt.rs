use core::ptr::addr_of;

use log::debug;
use x86_64::{
    instructions::tables::load_tss,
    registers::segmentation::{Segment, CS, SS},
    structures::gdt::{Descriptor, SegmentSelector},
    VirtAddr,
};

use crate::{arch::x86::cpu::PCRS, memory::address::VirtualAddress};

use super::cpu::{current_pcr_mut, MAX_CPUS};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const IA32_GS_BASE: u32 = 0xC0000101;
pub const IA32_KERNEL_GS_BASE: u32 = 0xC0000102;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Selectors {
    pub kernel_code_selector: SegmentSelector,
    pub kernel_data_selector: SegmentSelector,
    pub user_code_selector: SegmentSelector,
    pub user_data_selector: SegmentSelector,
}

const STACK_SIZE: usize = 4096 * 5;
static CPU_INTERRUPT_STACKS: [[u8; STACK_SIZE]; MAX_CPUS] = [[0; STACK_SIZE]; MAX_CPUS];
static CPU_KERNEL_STACKS: [[u8; STACK_SIZE]; MAX_CPUS] = [[0; STACK_SIZE]; MAX_CPUS];

pub fn init(cpu_id: u32) {
    unsafe {
        let pcr = PCRS.get_mut(cpu_id as usize).unwrap();

        let kernel_code_selector = pcr.gdt.append(Descriptor::kernel_code_segment());
        let kernel_data_selector = pcr.gdt.append(Descriptor::kernel_data_segment());
        let user_data_selector = pcr.gdt.append(Descriptor::user_data_segment());
        let user_code_selector = pcr.gdt.append(Descriptor::user_code_segment());

        pcr.tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            let stack_start = VirtAddr::from_ptr(addr_of!(CPU_INTERRUPT_STACKS[cpu_id as usize]));
            stack_start + STACK_SIZE as u64
        };

        pcr.tss.privilege_stack_table[0] = {
            let stack_start = VirtAddr::from_ptr(addr_of!(CPU_KERNEL_STACKS[cpu_id as usize]));
            stack_start + STACK_SIZE as u64
        };

        let tss_selector = pcr.gdt.append(Descriptor::tss_segment(&pcr.tss));

        pcr.gdt.load();

        CS::set_reg(kernel_code_selector);
        SS::set_reg(kernel_data_selector);
        load_tss(tss_selector);

        let selectors = Selectors {
            kernel_code_selector,
            kernel_data_selector,
            user_code_selector,
            user_data_selector,
        };
        pcr.selectors = Some(selectors);
    }

    debug!("GDT loaded");
}

pub fn set_tss_kernel_stack(stack: VirtualAddress) {
    let pcr = current_pcr_mut();
    pcr.tss.privilege_stack_table[0] = VirtAddr::new(stack.as_u64());
    pcr.kernel_rsp = stack.as_u64();
}
