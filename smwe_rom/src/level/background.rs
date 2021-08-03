use crate::{compression::lc_rle1, error::DecompressionError};

pub type BackgroundTileID = u8;

#[derive(Clone)]
pub struct BackgroundData {
    tile_ids: Vec<BackgroundTileID>,
}

impl BackgroundData {
    pub fn read_from(input: &[u8]) -> Result<Self, DecompressionError> {
        let tile_ids = lc_rle1::decompress(input)?;
        Ok(Self { tile_ids })
    }
}
