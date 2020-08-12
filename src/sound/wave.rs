//! This module implements the minimum machinery needed for wave audio parsing for playback on the GBA.
//! It does not strive to be complete or 100% correct, but "good enough".

use crate::FS;

use core::u16;
use core::u32;

use alloc::string::String;
use byte_slice_cast::AsSliceOf;
use gbfs_rs::GBFSError;
use tiny_riff::*;

/// Failure conditions that may arise when parsing a wav file for use on GBA.
#[derive(Debug, Clone)]
pub enum WaveError {
    /// Something went wrong when trying to open the GBFS file.
    File(GBFSError),
    /// The GBA only supports 8 bit signed PCM samples.
    UnsupportedSampleType,
    /// More than one sound channel is not supported.
    TooManyChannels,
    /// Something was wrong with the structure of the RIFF container.
    Riff(RiffError),
    /// The file does not contain chunk(s) expected in a wave file
    MissingChunk(String),
    /// The wave data can't be cast to a u32 slice required for playback on the GBA
    Cast(byte_slice_cast::Error),
}

impl core::fmt::Display for WaveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use WaveError::*;
        match self {
            File(err) => write!(f, "WaveError: Can't open file due to a GBFS error: {}", err),
            UnsupportedSampleType => {
                write!(f, "WaveError: Only 8 bit signed samples are supported")
            }
            TooManyChannels => write!(f, "WaveError: Only 1 channel is supported"),
            Riff(err) => write!(f, "WaveError: Can't open file due to a RIFF error: {}", err),
            MissingChunk(chunk) => write!(f, "WaveError: File is missing required chunk {}", chunk),
            Cast(err) => write!(
                f,
                "WaveError: Can't open file because data can't be cast to &[u32]: {}",
                err
            ),
        }
    }
}

impl From<RiffError> for WaveError {
    fn from(error: RiffError) -> Self {
        WaveError::Riff(error)
    }
}
impl From<GBFSError> for WaveError {
    fn from(error: GBFSError) -> Self {
        WaveError::File(error)
    }
}
impl From<byte_slice_cast::Error> for WaveError {
    fn from(error: byte_slice_cast::Error) -> Self {
        WaveError::Cast(error)
    }
}

/// `Wave` describes the structure of a wave file.
pub(super) struct Wave<'a> {
    /// The actual raw sample data, ready to be chucked into the GBA sound HW
    pub audio: &'a [u32],
    /// Samplerate, in Hz
    pub sample_rate: u32,
}

impl Wave<'_> {
    /// Load from a GBFS file.
    pub fn from_file(name: &str) -> Result<Wave, WaveError> {
        let data = FS.get_file_data_by_name(name)?;
        let riff = RiffReader::new(data);

        let wave_chunk =
            match riff.get_chunk(ChunkId::from_ascii([b'R', b'I', b'F', b'F']).unwrap()) {
                Some(chunk) => chunk,
                None => return Err(WaveError::MissingChunk(String::from("RIFF"))),
            }?;

        // Within the top-level chunk there must be a chunk containing format info
        let sc = RiffReader::new(wave_chunk.data);
        let fmt = match sc.get_chunk(ChunkId::from_ascii([b'f', b'm', b't', b'\0']).unwrap()) {
            Some(chunk) => chunk?,
            None => return Err(WaveError::MissingChunk(String::from("fmt "))),
        };

        // Validate the the file's format is something we can work with
        let format_type = u16::from_le_bytes([fmt.data[0], fmt.data[1]]);
        // We only support PCM audio
        if format_type != 1 {
            return Err(WaveError::UnsupportedSampleType);
        }
        let num_chans = u16::from_le_bytes([fmt.data[2], fmt.data[3]]);
        if num_chans != 1 {
            return Err(WaveError::TooManyChannels);
        }
        let sample_rate = u32::from_le_bytes([fmt.data[4], fmt.data[5], fmt.data[6], fmt.data[7]]);
        // We don't care about the next 2 data points, skip
        let bits_per_sample = u16::from_le_bytes([fmt.data[14], fmt.data[15]]);
        if bits_per_sample != 8 {
            return Err(WaveError::UnsupportedSampleType);
        }

        // We don't care about optional chunks, just take us to the data
        let wave = match sc.get_chunk(ChunkId::from_ascii([b'd', b'a', b't', b'a']).unwrap()) {
            Some(chunk) => chunk?,
            None => return Err(WaveError::MissingChunk(String::from("data"))),
        };

        // The borrow checker is not yet smart enough to reason about the fact that all the RIFF processing structs only ever
        // subslice slices from FS (which has a static lifetime), meaning that this cast should be safe.
        // Of course, this makes the risky assumptions that 1) FS is 'static and 2) The implementation of tiny_riff never changes to copy data.
        // TODO: Design an API for RIFF that returns the indices into the slice rather than the slice itself to make this unnecessary
        unsafe {
            let wave_data =
                core::slice::from_raw_parts::<'static, u8>(wave.data.as_ptr(), wave.data.len());
            return Ok(Wave {
                sample_rate,
                audio: wave_data.as_slice_of::<u32>()?,
            });
        }
    }
}
