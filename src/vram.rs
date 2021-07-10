//! Abstractions for interacting with VRAM that are missing from the `gba` crate.
//! TODO: Fix the situation upstream. This is stuff that basically every game needs.

use core::convert::TryInto;
use core::ptr;
use gba::prelude::*;

/// Address at which VRAM starts.
const VRAM_BASE_ADDR: usize = 0x600_0000;
/// Size of a screenblock in bytes.
pub const SCREENBLOCK_SIZE_BYTES: usize = 32 * 32 * 2;
/// Number of u16's that make up a screenblock.
const SCREENBLOCK_SIZE_U16S: usize = SCREENBLOCK_SIZE_BYTES / 2;
/// Size of a charblock in bytes.
const CHARBLOCK_SIZE_BYTES: usize = 16 * 1024;
/// Size of a charblock in 4bpp tiles.
const CHARBLOCK_SIZE_4BPP_TILES: usize = CHARBLOCK_SIZE_BYTES / 4 / 8;
/// Number of u32's that make up a 4bpp tile.
pub const TILE4BPP_NUM_U32S: usize = 8;

/// A charblock ID.
pub type CharblockID = usize;
/// The largest permitted charblock ID.
pub const CHARBLOCK_ID_MAX: CharblockID = 3;
/// A screenblock ID.
pub type ScreenblockID = usize;
/// The largest permitted screenblock ID.
pub const SCREENBLOCK_ID_MAX: ScreenblockID = 31;

/// The number of 4bpp tiles that fit into a single charblock.

/// Returned on failed interactions with VRAM.
#[derive(Debug, Copy, Clone)]
pub enum VRAMError {
    InvalidID,
    InvalidOffset,
    InvalidSize,
}

/// DMA transfer tiles into the given charblock.
///
/// `offset` is given in number of 4bpp tiles, not u32's or bytes.
///
/// # Errors
/// Will return a `VRAMError` if the charblock ID is invalid or the offset is so large that the write would be out of charblock bounds.
///
/// # Safety
///
/// It's up to you to ensure there are no overlapping screenblocks in the VRAM that will be used.
/// TODO: Should functions using DMA be marked unsafe? Not reacting to interrupts can lead to spooky action at a distance.
pub fn copy_tiles_4bpp_dma(
    data: &[u32],
    dest_charblock: CharblockID,
    offset: usize,
) -> Result<(), VRAMError> {
    // Sanity checks
    if dest_charblock > CHARBLOCK_ID_MAX {
        return Err(VRAMError::InvalidID);
    }
    if offset > CHARBLOCK_SIZE_4BPP_TILES {
        return Err(VRAMError::InvalidOffset);
    }
    if (offset + (data.len() / CHARBLOCK_SIZE_4BPP_TILES)) > CHARBLOCK_SIZE_4BPP_TILES {
        return Err(VRAMError::InvalidSize);
    }
    if (data.len() % TILE4BPP_NUM_U32S) != 0 {
        return Err(VRAMError::InvalidSize);
    }

    // Transfer
    let dest_addr: usize =
        VRAM_BASE_ADDR + (dest_charblock * CHARBLOCK_SIZE_BYTES) + (offset * TILE4BPP_NUM_U32S);
    let dest_len: u16 = data.len().try_into().unwrap();
    unsafe {
        DMA3SAD.write(data.as_ptr() as usize);
        DMA3DAD.write(dest_addr);
        DMA3CNT_L.write(dest_len);
        DMA3CNT_H.write(DmaControl::new().with_enabled(true).with_transfer_u32(true));
    }
    return Ok(());
}

/// DMA transfer tilemap into the given screenblock.
///
/// # Errors
/// Will return a `VRAMError` if the screenblock ID is invalid,
/// or the data is so large that the write would be out of screenblock bounds.
///
/// # Safety
/// It's up to you to ensure there are no overlapping charblocks in the VRAM that will be used.
/// TODO: Should functions using DMA be marked unsafe? Not reacting to interrupts can lead to spooky action at a distance.
pub fn copy_tilemap_dma(data: &[u16], dest_screenblock: ScreenblockID) -> Result<(), VRAMError> {
    // Sanity checks
    if dest_screenblock > SCREENBLOCK_ID_MAX {
        return Err(VRAMError::InvalidID);
    }
    if (data.len()) > SCREENBLOCK_SIZE_U16S {
        return Err(VRAMError::InvalidSize);
    }

    // Transfer
    let dest_addr: usize = VRAM_BASE_ADDR + (dest_screenblock * SCREENBLOCK_SIZE_BYTES);
    let dest_len: u16 = data.len().try_into().unwrap();
    unsafe {
        DMA3SAD.write(data.as_ptr() as usize);
        DMA3DAD.write(dest_addr);
        DMA3CNT_L.write(dest_len);
        DMA3CNT_H.write(
            DmaControl::new()
                .with_enabled(true)
                .with_transfer_u32(false),
        );
    }
    return Ok(());
}

/// Use DMA to fill an entire screenblock with a reference to the same tile.
///
/// # Safety
/// It's up to you to ensure there are no overlapping charblocks in the VRAM that will be used.
pub fn fill_tilemap_dma(entry: u16, dest_screenblock: ScreenblockID) -> Result<(), VRAMError> {
    // Sanity checks
    if dest_screenblock > SCREENBLOCK_ID_MAX {
        return Err(VRAMError::InvalidID);
    }
    // Transfer
    let dest_addr: usize = VRAM_BASE_ADDR + (dest_screenblock * SCREENBLOCK_SIZE_BYTES);
    let src: [u16; 1] = [entry];
    unsafe {
        DMA3SAD.write(src.as_ptr() as usize);
        DMA3DAD.write(dest_addr);
        DMA3CNT_L.write(SCREENBLOCK_SIZE_U16S as u16);
        DMA3CNT_H.write(
            DmaControl::new()
                .with_enabled(true)
                .with_transfer_u32(false)
                .with_src_addr(SrcAddrControl::Fixed),
        );
    }
    return Ok(());
}

/// Set the tilemap entry at index to the given value.
///
/// # Errors
/// Will return a `DMAError` if the screenblock ID or tilemap index is invalid.
///
/// # Safety
/// It's up to you to ensure there are no overlapping charblocks in the VRAM that will be used.
#[inline]
pub fn set_tilemap_entry(
    entry: u16,
    dest_screenblock: ScreenblockID,
    tilemap_idx: usize,
) -> Result<(), VRAMError> {
    // Sanity checks
    if dest_screenblock > SCREENBLOCK_ID_MAX {
        return Err(VRAMError::InvalidID);
    }
    if tilemap_idx > SCREENBLOCK_SIZE_U16S {
        return Err(VRAMError::InvalidOffset);
    }

    // Write
    unsafe {
        let dst: *mut u16 = (VRAM_BASE_ADDR
            + (dest_screenblock * SCREENBLOCK_SIZE_BYTES)
            + (tilemap_idx * 2)) as *mut u16;
        ptr::write_volatile(dst, entry);
    }
    return Ok(());
}
