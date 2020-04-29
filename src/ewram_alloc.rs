use core::alloc::*;
use core::mem;

use gba;

pub const EWRAM_BASE: usize = 0x200_0000;
pub const EWRAM_END: usize = 0x203_FFFF;
pub const EWRAM_SIZE: usize = EWRAM_END - EWRAM_BASE;
pub struct MyBigAllocator; // Empty struct; doesn't actually save any data, it's just a handle that can be made immutably `static` to satisfy the global_alloc interface

struct BlockAllocate {
    free: bool,
    size: usize,
}

unsafe impl GlobalAlloc for MyBigAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // TODO: Philipp, kannst du sagen ob dir meine Kommentare Sinn ergeben?
        // Current block we're checking for allocation eligibility
        let mut current_block_position = EWRAM_BASE;
        let mut current_block: &mut BlockAllocate;
        while current_block_position < EWRAM_END {
            // Obtain a reference to the block we're checking
            current_block = mem::transmute::<usize, &mut BlockAllocate>(current_block_position);
            // Check whether data + control structure would fit
            if current_block.free
                && current_block.size - mem::size_of::<BlockAllocate>() >= layout.size()
            {
                // big enough free block found
                // lets split it
                let old_size = current_block.size;

                // new size is allocated Bytes + size of Control Block\
                current_block.size = layout.size() + mem::size_of::<BlockAllocate>(); // layout.size bytes of data + control bytes
                current_block.free = false;
                create_new_block(
                    current_block_position + current_block.size,
                    old_size - current_block.size,
                );
                gba::debug!(
                    "allocate Block at {} with size {}",
                    current_block_position,
                    current_block.size
                );
                return mem::transmute::<usize, *mut u8>(
                    current_block_position + mem::size_of::<BlockAllocate>(),
                );
            }
            // Check next block
            // We have to move layout.size() + mem::size_of::<BlockAllocate>() positions
            current_block_position = current_block_position + current_block.size;
        }
        // If no block could be found, return a null pointer
        return mem::transmute::<usize, *mut u8>(0);
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let test: &mut BlockAllocate = mem::transmute::<usize, &mut BlockAllocate>(
            mem::transmute::<*mut u8, usize>(ptr) - mem::size_of::<BlockAllocate>(),
        );
        test.free = true;
        // TODO: Check whether adjacent blocks are free and perform coalescing
    }
}

/// Allocate block of `size` at address `base`
pub unsafe fn create_new_block(base: usize, size: usize) {
    gba::debug!("New free Block with size {} on Mem: {}", size, base);
    let test: &mut BlockAllocate = mem::transmute::<usize, &mut BlockAllocate>(base);
    test.size = size;
    test.free = true;
}
