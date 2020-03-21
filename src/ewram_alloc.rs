use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr::NonNull;

/// This module implements a simple allocator for external work RAM.
/// The code is largely based on the linked_list_allocator crate, modified to work without CPU atomic support.
/// WARNING: EWRAM is slow. Only use for allocations where this is acceptable.
use linked_list_allocator::Heap;

// TODO: Consider moving this stuff into gba crate (there's an open issue for that)
pub(crate) const EWRAM_BASE: usize = 0x200_0000;
pub(crate) const EWRAM_END: usize = 0x203_FFFF;
pub(crate) const EWRAM_SIZE: usize = EWRAM_END - EWRAM_BASE;
/// A heap implementing GlobalAlloc without using a lock (useful on no_std platforms without atomics, as a spin lock is impossible to implement there).
/// # Safety
/// Only use on platforms where there's no other choice!
/// This is ONLY to be used in a single-threaded, single-CPU, `no_std` context! Race conditions galore!
pub(crate) struct RaceyHeap(UnsafeCell<Heap>);

impl RaceyHeap {
    /// Creates an empty heap. All allocate calls will return `None`.
    /// This is primarily useful for having a `static`-friendly object, which the `global_alloc` interface requires.
    /// In order for the allocator to actually be usable, you have to call `init()`.
    pub const fn empty() -> RaceyHeap {
        RaceyHeap(UnsafeCell::new(Heap::empty()))
    }

    /// Configures the heap to actually be usable.
    /// This has to be separate from the constructor, because this function is not `static`.
    ///
    /// # Safety
    ///
    /// This function must be called at most once and must only be used on an empty heap.
    pub(crate) unsafe fn init(&self, heap_start: usize, heap_size: usize) {
        &self.0.get().as_mut().unwrap().init(heap_start, heap_size);
    }
}

unsafe impl GlobalAlloc for RaceyHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0
            .get()
            .as_mut()
            .unwrap()
            .allocate_first_fit(layout)
            .ok()
            .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0
            .get()
            .as_mut()
            .unwrap()
            .deallocate(NonNull::new_unchecked(ptr), layout)
    }
}

/// This Sync "implementation" is incorrect and wildly unsafe if actually used.
/// That doesn't matter here though, as the GBA is a single-processor, single-thread-of-execution system.
/// This trait just needs to be here to satisfy the compiler, as it seems to lack a way to tell it that the target
/// machine has no such thing as concurrent execution.
unsafe impl Sync for RaceyHeap {}
