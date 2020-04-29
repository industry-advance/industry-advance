use core::alloc::*;
use core::mem;

use core::ptr;

use gba;

pub const EWRAM_BASE: usize = 0x200_0000;
pub const EWRAM_END: usize = 0x203_3FF0;
// Something seems to be in 0x203FFF
// pub const EWRAM_END: usize = 0x203_FFF0;
pub const EWRAM_SIZE: usize = EWRAM_END - EWRAM_BASE;
pub struct MyBigAllocator; // Empty struct; doesn't actually save any data, it's just a handle that can be made immutably `static` to satisfy the global_alloc interface

#[derive(Debug, Clone)]
#[repr(C)]
pub struct BlockAllocate {
    size: usize,  // 4Bytes
    marker: u16,  // 2Bytes
    free: bool,   // 1Byte
    filler: bool, // 1Byte
}

unsafe impl GlobalAlloc for MyBigAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        gba::debug!("Alloc Call, find {} Bytes", layout.size());
        // TODO: Philipp, kannst du sagen ob dir meine Kommentare Sinn ergeben?
        // Current block we're checking for allocation eligibility
        let mut current_block_position = EWRAM_BASE;
        while current_block_position < EWRAM_END {
            gba::debug!("Check Block on position 0x{:x}", current_block_position);
            // Obtain a reference to the block we're checking
            let mut current_block: BlockAllocate =
                ptr::read_volatile::<BlockAllocate>(current_block_position as *const BlockAllocate);
            gba::debug!("Block Metadata {:?}", current_block);
            assert!(current_block.size != 0);
            // Check whether data + control structure would fit
            if current_block.free
                && current_block.size - mem::size_of::<BlockAllocate>() >= layout.size()
            {
                // big enough free block found
                // lets split it
                let old_size = current_block.size;

                // new size is allocated Bytes + size of Control Block\
                current_block.size =
                    layout.size() + mem::size_of::<BlockAllocate>() + (4 - layout.size() % 4); // layout.size bytes of data + control bytes
                current_block.free = false;
                assert_eq!(
                    (old_size - current_block.size) + current_block.size,
                    old_size
                );

                create_new_block(
                    current_block_position + current_block.size,
                    old_size - current_block.size,
                );
                gba::debug!(
                    "allocate Block at 0x{:x} with size {}",
                    current_block_position,
                    current_block.size
                );
                ptr::write_volatile::<BlockAllocate>(
                    current_block_position as *mut BlockAllocate,
                    current_block.clone(),
                );
                
                return (current_block_position + mem::size_of::<BlockAllocate>()) as *mut u8;
            }
            // Check next block
            // We have to move layout.size() + mem::size_of::<BlockAllocate>() positions
            current_block_position = current_block_position + current_block.size;
        }
        // If no block could be found, return a null pointer
        return 0 as *mut u8;
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut current_block: BlockAllocate = ptr::read_volatile::<BlockAllocate>(
            ((ptr as usize) - mem::size_of::<BlockAllocate>()) as *const BlockAllocate,
        );

        current_block.free = true;
        ptr::write_volatile::<BlockAllocate>(
            ((ptr as usize) - mem::size_of::<BlockAllocate>()) as *mut BlockAllocate,
            current_block.clone(),
        );

        gba::debug!("Deallocation {} Bytes at 0x{:p}", layout.size(), ptr);
        // TODO: Check whether adjacent blocks are free and perform coalescing
    }
}

/// Allocate block of `size` at address `base`
pub fn create_new_block(base: usize, size: usize) {
    gba::debug!(
        "New free Block with size {} on Mem: 0x{:x}",
        size.clone(),
        base.clone()
    );
    let control = BlockAllocate {
        size: size,
        marker: 0xDEAD,
        free: true,
        filler: false,
    };
    //let control_block: &mut BlockAllocate = mem::transmute::<usize, &mut BlockAllocate>(base);
    gba::debug!("Size Test 1 {}", size);
    gba::debug!("Size in Control Block: {}", control.size);

    let c2: BlockAllocate;
    let pointer: *mut BlockAllocate = base as *mut BlockAllocate;
    gba::debug!("Pointer To Write Base: {:p}", pointer);
    unsafe {
        ptr::write_volatile(pointer, control);

        c2 = ptr::read_volatile::<BlockAllocate>(pointer);
    }
    gba::debug!("Pointer To Write Base (AW): {:p}", pointer);

    gba::debug!("C2 Perspective");
    //gba::debug!("Position of size var 0x{:p}", &c2.size);
    gba::debug!("Size: {}", c2.size.clone());
    //gba::debug!("Position of free var 0x{:p}", &c2.free);
    gba::debug!("Free: {}", c2.free.clone());
    //assert_eq!(c2.size, size);
    //assert_eq!(control_block.size, size);
    //assert_eq!(control.size, c2.size.clone())
}
