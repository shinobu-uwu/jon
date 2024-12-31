use alloc::{string::String, vec::Vec};
use bitmap_allocator::{BitAlloc, BitAlloc64K};
use lazy_static::lazy_static;
use log::debug;
use scheduler::Scheduler;
use spinning_top::Spinlock;

pub mod pid;
pub mod scheduler;
pub mod task;

pub static SCHEDULER: Spinlock<Scheduler> = Spinlock::new(Scheduler::new());
lazy_static! {
    pub static ref PID_ALLOCATOR: Spinlock<BitAlloc64K> = {
        let mut bitmap = BitAlloc64K::default();
        bitmap.insert(0..BitAlloc64K::CAP); // marks all bits as available
        bitmap.remove(0..1); // marks PID 0 as used, for the kernel

        Spinlock::new(bitmap)
    };
}

pub fn debug_bitmap() {
    let allocator = PID_ALLOCATOR.lock();

    // Count used and free PIDs
    let mut used = 0;
    let mut free = 0;
    let mut used_pids = Vec::new();

    for i in 0..65536 {
        // BitAlloc64K size
        if allocator.test(i) {
            used += 1;
            used_pids.push(i);
        } else {
            free += 1;
        }
    }

    debug!(
        "BitAlloc Status:\n\
        Total PIDs: 65536\n\
        Used PIDs: {}\n\
        Free PIDs: {}\n\
        Usage: {:.2}%\n\
        First 10 used PIDs: {:?}\n\
        First free PID: {}",
        used,
        free,
        (used as f32 / 65536.0) * 100.0,
        &used_pids.iter().take(10).collect::<Vec<_>>(),
        (0..65536).find(|&i| !allocator.test(i)).unwrap_or(65536)
    );
}

// Or a more detailed version that shows ranges of used/free PIDs
pub fn debug_bitmap_detailed() {
    let allocator = PID_ALLOCATOR.lock();

    let mut ranges = Vec::new();
    let mut start = 0;
    let mut prev_state = allocator.test(0);

    for i in 1..65536 {
        let current_state = allocator.test(i);
        if current_state != prev_state {
            ranges.push((start..i, prev_state));
            start = i;
            prev_state = current_state;
        }
    }
    ranges.push((start..65536, prev_state));

    debug!("BitAlloc Ranges:");
    for (range, is_used) in ranges {
        if range.end - range.start > 1 {
            debug!(
                "{}: PIDs {}-{} ({} PIDs)",
                if is_used { "Used" } else { "Free" },
                range.start,
                range.end - 1,
                range.end - range.start
            );
        } else {
            debug!(
                "{}: PID {}",
                if is_used { "Used" } else { "Free" },
                range.start
            );
        }
    }
}

// Or a visual representation
pub fn debug_bitmap_visual() {
    let allocator = PID_ALLOCATOR.lock();

    debug!("BitAlloc Visual Map (■=used, □=free):");
    for y in 0..16 {
        // Show first 256 PIDs in 16x16 grid
        let mut line = String::new();
        for x in 0..16 {
            let pid = y * 16 + x;
            line.push(if allocator.test(pid) { '■' } else { '□' });
        }
        debug!("{}", line);
    }
}

// Combine them into one comprehensive debug function
pub fn debug_allocator() {
    debug!("=== PID Allocator Debug Information ===");
    debug_bitmap();
    debug!("\n=== Detailed Range Information ===");
    debug_bitmap_detailed();
    debug!("\n=== Visual Representation ===");
    debug_bitmap_visual();
}
