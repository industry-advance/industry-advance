use gbfs_rs::GBFSError;
/// The error returned in sprite allocation-related failure cases.
#[derive(Clone, Debug, PartialEq)]
pub enum HWSpriteAllocError {
    OAMFull,
    VRAMFull,
    File(GBFSError),
    SpriteVisibilityStackEmpty,
}

impl core::fmt::Display for HWSpriteAllocError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use HWSpriteAllocError::*;
        match self {
            OAMFull => write!(
                f,
                "HWSpriteAllocError: Can't create sprite because OAM is full"
            ),
            VRAMFull => write!(
                f,
                "HWSpriteAllocError: No contiguous free block of VRAM available to allocate hardware sprite"
            ),
            File(gbfs_err) => write!(
                f,
                "HWSpriteAllocError: Couldn't read sprite file: {}",
                gbfs_err
            ),
            SpriteVisibilityStackEmpty => write!(
                f,
                "HWSpriteAllocError: No sprite visibility info was saved before the call"
            ),
        }
    }
}
