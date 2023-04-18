use thiserror::Error;

use crate::{objects::map16::Tile8x8, AddrSnes, DataBlock, DataKind, RomDisassembly, SnesSlice};

// -------------------------------------------------------------------------------------------------

#[derive(Debug, Error)]
#[error("Could not parse GFX list at:\n- {0}")]
pub struct ObjectGfxListParseError(pub SnesSlice);

// -------------------------------------------------------------------------------------------------

const OBJECT_GFX_LIST: SnesSlice = SnesSlice::new(AddrSnes(0x00A92B), 26 * 4);

// -------------------------------------------------------------------------------------------------

pub struct ObjectGfxList {
    gfx_file_nums: Vec<u8>,
}

// -------------------------------------------------------------------------------------------------

impl ObjectGfxList {
    pub fn parse(disasm: &mut RomDisassembly) -> Result<Self, ObjectGfxListParseError> {
        let block = DataBlock { slice: OBJECT_GFX_LIST, kind: DataKind::GfxListObjects };
        let gfx_file_nums =
            disasm.rom_slice_at_block(block, |_| ObjectGfxListParseError(OBJECT_GFX_LIST))?.as_bytes()?.to_vec();
        Ok(Self { gfx_file_nums })
    }

    pub fn gfx_file_for_object_tile(&self, tile: Tile8x8, tileset: usize) -> usize {
        let idx = (tileset * 4) + tile.layer();
        self.gfx_file_nums[idx] as usize
    }
}
