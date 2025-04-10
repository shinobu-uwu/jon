use core::mem::offset_of;
use core::ptr::addr_of;
use core::{arch, usize};

use log::{debug, info};
use x86_64::instructions::tables::load_tss;
use x86_64::registers::model_specific::{GsBase, KernelGsBase, Msr};
use x86_64::registers::segmentation::{Segment, CS, GS, SS};
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

use crate::memory::address::VirtualAddress;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
const IA32_GS_BASE: u32 = 0xC0000101;
const IA32_KERNEL_GS_BASE: u32 = 0xC0000102;

pub static mut TSS: TaskStateSegment = TaskStateSegment::new();
static mut PCR: ProcessorControlRegion = ProcessorControlRegion {
    user_rsp: 0,
    kernel_rsp: 0,
};

pub static mut GDT: (GlobalDescriptorTable, Selectors) = {
    let mut gdt = GlobalDescriptorTable::new();
    let kernel_code_selector = gdt.append(Descriptor::kernel_code_segment());
    let kernel_data_selector = gdt.append(Descriptor::kernel_data_segment());
    let user_data_selector = gdt.append(Descriptor::user_data_segment());
    let user_code_selector = gdt.append(Descriptor::user_code_segment());

    (
        gdt,
        Selectors {
            kernel_code_selector,
            kernel_data_selector,
            user_code_selector,
            user_data_selector,
            tss_selector: None,
        },
    )
};

#[repr(C)]
pub struct ProcessorControlRegion {
    pub user_rsp: u64,
    pub kernel_rsp: u64,
}

#[repr(C)]
pub struct Selectors {
    pub kernel_code_selector: SegmentSelector,
    pub kernel_data_selector: SegmentSelector,
    pub user_code_selector: SegmentSelector,
    pub user_data_selector: SegmentSelector,
    pub tss_selector: Option<SegmentSelector>,
}

pub fn init() {
    unsafe {
        TSS.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(addr_of!(STACK));
            stack_start + STACK_SIZE as u64
        };

        TSS.privilege_stack_table[0] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(addr_of!(STACK));
            stack_start + STACK_SIZE as u64
        };

        let tss_selector = GDT.0.append(Descriptor::tss_segment(&TSS));

        GDT.0.load();

        CS::set_reg(GDT.1.kernel_code_selector);
        SS::set_reg(GDT.1.kernel_data_selector);
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
    }

    debug!("GDT loaded");
}

pub fn set_tss_kernel_stack(stack: VirtualAddress) {
    unsafe {
        TSS.privilege_stack_table[0] = VirtAddr::new(stack.as_u64());
        PCR.kernel_rsp = stack.as_u64();
    }
}
