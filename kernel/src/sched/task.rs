use core::arch::asm;

use bitmap_allocator::BitAlloc;
use log::debug;

use crate::{
    arch::x86::{
        gdt::GDT,
        memory::{PMM, VMM},
    },
    memory::{
        address::VirtualAddress, paging::PageFlags, physical::PhysicalMemoryManager, stack::Stack,
        PAGE_SIZE,
    },
    sched::{pid::Pid, PID_ALLOCATOR},
};

const STACK_START: usize = 0xffff888000000000;
const STACK_SIZE: usize = 0x4000; // 16 KiB

#[derive(Debug)]
pub struct Task {
    pid: Pid,
    kernel_stack: Stack,
    user_stack: Stack,
    context: Context,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct Context {
    // Callee-saved registers
    rsp: u64, // Stack pointer
    // Return address for context switch
    rip: u64,
    rbp: u64, // Base pointer
    rbx: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
}

impl Task {
    pub fn new() -> Self {
        let pid = Pid::new(PID_ALLOCATOR.lock().alloc().unwrap());
        debug!("Creating task with PID {}", pid);
        let kernel_stack = Stack::new(
            VirtualAddress::new(STACK_START + pid.as_usize() * STACK_SIZE),
            STACK_SIZE,
        );
        let user_stack_addr = VirtualAddress::new(0x7FFF_FFFF_0000);
        let user_stack = Stack::new(user_stack_addr, STACK_SIZE);
        let mut context = Context::default();
        let bin_addr = load_binary(include_bytes!("../bin/task"));
        let bin_flags = VMM.lock().page_flags(bin_addr).unwrap();
        debug!("Binary at {:#x?} with flags {:#x?}", bin_addr, bin_flags);

        context.rsp = user_stack.top().as_u64();
        context.rip = bin_addr.as_u64();

        Self {
            pid,
            kernel_stack,
            user_stack,
            context,
        }
    }

    #[inline(always)]
    #[no_mangle]
    pub unsafe fn restore(&self) -> ! {
        debug!("Restoring task {:#x?}", self,);
        asm!(
            "mov ds, [{gdt} + 6]",
            "mov es, [{gdt} + 6]",
            "mov fs, [{gdt} + 6]",
            "mov gs, [{gdt} + 6]", // SS is handled by iret
            // setup the stack frame iret expects
            "push [{gdt} + 6]", // data selector
            "push [{context}]",          // stack pointer
            "pushf",
            "push [{gdt} + 4]", // code selector
            "push [{context} + 8]", // instruction pointer
            "iretq",
            gdt = in(reg) &GDT.1, // each selector is 16 bits
            context = in(reg) &self.context,
            options(noreturn)
        );
    }
}

fn load_binary(binary: &[u8]) -> VirtualAddress {
    let pages_needed = (binary.len() + PAGE_SIZE - 1) / PAGE_SIZE;

    // Allocate physical memory for the binary
    let phys_addr = PMM
        .lock()
        .allocate_contiguous(pages_needed * PAGE_SIZE)
        .unwrap();

    // Map it into user space with appropriate permissions
    let user_virt_addr = VirtualAddress::new(0x400000); // Common starting point for user programs
    VMM.lock()
        .map_range(
            user_virt_addr,
            phys_addr,
            pages_needed * PAGE_SIZE,
            PageFlags::PRESENT | PageFlags::USER_ACCESSIBLE | PageFlags::WRITABLE,
        )
        .unwrap();

    // Copy binary data to the mapped region
    unsafe {
        core::ptr::copy_nonoverlapping(
            binary.as_ptr(),
            user_virt_addr.as_usize() as *mut u8,
            binary.len(),
        );
    }

    user_virt_addr
}
