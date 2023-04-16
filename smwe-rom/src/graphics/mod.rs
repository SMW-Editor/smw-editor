use thiserror::Error;

use crate::{
    disassembler::RomDisassembly,
    graphics::gfx_file::{GfxFile, Tile, GFX_FILES_META},
    snes_utils::addr::AddrSnes,
    RegionCode,
    RomInternalHeader,
    RomParseError,
};

pub mod color;
pub mod gfx_file;
pub mod palette;

// -------------------------------------------------------------------------------------------------

#[derive(Debug, Error)]
#[error("Cannot get GFX tile at WRAM address ${0:X}")]
pub struct TileFromWramError(u32);

// -------------------------------------------------------------------------------------------------

pub struct Gfx {
    pub files: Vec<GfxFile>,
}

// -------------------------------------------------------------------------------------------------

impl Gfx {
    pub fn parse_files(
        disasm: &mut RomDisassembly, internal_header: &RomInternalHeader,
    ) -> Result<Self, RomParseError> {
        let revised_gfx =
            matches!(internal_header.region_code, RegionCode::Japan) || internal_header.version_number > 0;
        let mut files = Vec::with_capacity(GFX_FILES_META.len());
        for file_num in 0..GFX_FILES_META.len() {
            let file = GfxFile::new(disasm, file_num, revised_gfx).map_err(|e| RomParseError::GfxFile(file_num, e))?;
            files.push(file);
        }
        Ok(Self { files })
    }

    pub fn tile16x16_from_wram(&self, wram_addr: AddrSnes) -> Result<&[Tile], TileFromWramError> {
        let (file, offset) = match wram_addr {
            // Mario graphics & berry animation
            AddrSnes(addr @ 0x7E2000..=0x7E7CFF) => (&self.files[0x32], addr - 0x7E2000),
            // Animated tiles
            AddrSnes(addr @ 0x7E7D00..=0x7EACFF) => (&self.files[0x33], addr - 0x7E7D00),
            // Unknown
            AddrSnes(addr) => return Err(TileFromWramError(addr)),
        };
        let index = offset as usize / file.tile_format.tile_size();
        Ok(&file.tiles[index..index + 4])
    }
}
