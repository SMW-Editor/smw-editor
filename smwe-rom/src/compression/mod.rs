pub mod lc_lz2;
pub mod lc_rle1;

use thiserror::Error;

pub use self::{lc_lz2::LcLz2Error, lc_rle1::LcRle1Error};

// -------------------------------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum DecompressionError {
    #[error("Decompression with LC-LZ2:\n- {0}")]
    LcLz2(LcLz2Error),
    #[error("Decompression with LC-RLE1:\n- {0}")]
    LcRle1(LcRle1Error),
}

// -------------------------------------------------------------------------------------------------

impl From<LcLz2Error> for DecompressionError {
    fn from(e: LcLz2Error) -> Self {
        DecompressionError::LcLz2(e)
    }
}

impl From<LcRle1Error> for DecompressionError {
    fn from(e: LcRle1Error) -> Self {
        DecompressionError::LcRle1(e)
    }
}
