//! This module implements the minimum machinery needed for wave audio parsing for playback on the GBA.
//! It does not strive to be complete or 100% correct, but "good enough".

use super::mixer::SAMPLE_RATE;
use crate::FS;

use core::u16;
use core::u32;

use byte_slice_cast::AsSliceOf;
use gbfs_rs::GBFSError;

/// Failure conditions that may arise when parsing a wav file for use on GBA.
#[derive(Debug, Clone)]
pub enum WaveError {
    /// Something went wrong when trying to open the GBFS file.
    File(GBFSError),
    /// The GBA only supports 8 bit signed PCM samples.
    UnsupportedSampleType,
    /// More than one sound channel is not supported.
    TooManyChannels,
    /// The sound has an unsupported sample rate
    UnsupportedSampleRate(u32),
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
            UnsupportedSampleRate(given_rate) => write!(
                f,
                "WaveError: Only a sample rate of {}Hz is supported (file is {}Hz)",
                SAMPLE_RATE, given_rate
            ),
            TooManyChannels => write!(f, "WaveError: Only 1 channel is supported"),
            Cast(err) => write!(
                f,
                "WaveError: Can't open file because data can't be cast to &[u32]: {}",
                err
            ),
        }
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

/// Loads wave audio data from a GBFS file.
///
/// Returns a slice of the portion of the file containing PCM samples, or an error.
pub fn from_file(name: &str) -> Result<&'static [i8], WaveError> {
    let data = FS.get_file_data_by_name(name)?;

    /*
    Because we only care about a small subset of the metadata which is available at the beginning of the file in fixed offsets,
    we don't use an actual RIFF parser here.
    Instead, just reading at those fixed offsets is good enough for our use case.
    */

    // Validate the the file's format is something we can work with
    let format_type = u16::from_le_bytes([data[20], data[21]]);
    // We only support PCM audio
    if format_type != 1 {
        return Err(WaveError::UnsupportedSampleType);
    }
    let num_chans = u16::from_le_bytes([data[22], data[23]]);
    if num_chans != 1 {
        return Err(WaveError::TooManyChannels);
    }
    let sample_rate = u32::from_le_bytes([data[24], data[25], data[26], data[27]]);
    if sample_rate != SAMPLE_RATE {
        return Err(WaveError::UnsupportedSampleRate(sample_rate));
    }
    let bits_per_sample = u16::from_le_bytes([data[34], data[35]]);
    if bits_per_sample != 8 {
        return Err(WaveError::UnsupportedSampleType);
    }

    // The header seems to be exactly 44 bytes large, meaning we can subslice that away to get at sound data
    let wave_data = &data[44..].as_slice_of::<i8>().unwrap();
    return Ok(wave_data);
}
