use crate::{
    error::GfxListParseError,
    objects::map16::Tile8x8,
    AddrSnes,
    DataBlock,
    DataKind,
    RomDisassembly,
    SnesSlice,
};

const OBJECT_GFX_LIST: SnesSlice = SnesSlice::new(AddrSnes(0x00A92B), 26 * 4);

pub struct ObjectGfxList {
    gfx_file_nums: Vec<u8>,
}

impl ObjectGfxList {
    pub fn parse(disasm: &mut RomDisassembly) -> Result<Self, GfxListParseError> {
        let block = DataBlock { slice: OBJECT_GFX_LIST, kind: DataKind::GfxListObjects };
        let gfx_file_nums =
            disasm.rom_slice_at_block(block, |_| GfxListParseError(OBJECT_GFX_LIST))?.as_bytes()?.to_vec();
        Ok(Self { gfx_file_nums })
    }

    pub fn gfx_file_for_object_tile(&self, tile: Tile8x8, tileset: usize) -> usize {
        let idx = (tileset * 4) + tile.layer();
        self.gfx_file_nums[idx] as usize
    }
}
