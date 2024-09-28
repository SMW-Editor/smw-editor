pub mod lc_lz2;
pub mod lc_rle1;

use duplicate::duplicate_item;
use paste::paste;
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

#[duplicate_item(algorithm; [LcLz2]; [LcRle1];)]
impl From<paste! { [<algorithm Error>] }> for DecompressionError {
    fn from(e: paste! { [<algorithm Error>] }) -> Self {
        DecompressionError::algorithm(e)
    }
}
