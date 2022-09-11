use crate::{error::GfxListParseError, AddrSnes, DataBlock, DataKind, RomDisassembly, SnesSlice};

const OBJECT_GFX_LIST: SnesSlice = SnesSlice::new(AddrSnes(0x00A94B), 0x68);

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
}
