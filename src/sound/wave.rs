//! This module implements the minimum machinery needed for wave audio parsing for playback on the GBA.
//! It does not strive to be complete or 100% correct, but "good enough".

use crate::FS;

use core::u16;
use core::u32;

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
        let data_u32 = FS.get_file_data_by_name_as_u32_slice(name)?;

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
        let bits_per_sample = u16::from_le_bytes([data[34], data[35]]);
        if bits_per_sample != 8 {
            return Err(WaveError::UnsupportedSampleType);
        }

        // FIXME: Figure out what's up with alignment

        // The header seems to be exactly 44 bytes large, meaning we can subslice that away to get at sound data
        //let wave_data = &data[44..];
        let wave_data = &data_u32[(44 / 4)..];
        return Ok(Wave {
            sample_rate,
            audio: wave_data,
        });
    }
}
