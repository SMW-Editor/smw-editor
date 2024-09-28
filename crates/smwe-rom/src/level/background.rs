use crate::compression::{lc_rle1, DecompressionError};

// -------------------------------------------------------------------------------------------------

pub type BackgroundTileID = u8;

#[derive(Debug, Clone)]
pub struct BackgroundData {
    _tile_ids: Vec<BackgroundTileID>,
}

// -------------------------------------------------------------------------------------------------

impl BackgroundData {
    /// Returns self and the number of bytes consumed by parsing.
    pub fn read_from(input: &[u8]) -> Result<(Self, usize), DecompressionError> {
        let (tile_ids, bytes_consumed) = lc_rle1::decompress(input)?;
        Ok((Self { _tile_ids: tile_ids }, bytes_consumed))
    }
}
