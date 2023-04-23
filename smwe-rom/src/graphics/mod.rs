use thiserror::Error;

use crate::{
    disassembler::RomDisassembly,
    graphics::{
        gfx_file::{GfxFile, Tile, GFX_FILES_META},
        palette::ColorPalettes,
    },
    level::Level,
    objects::{
        animated_tile_data::AnimatedTileData,
        map16::Block,
        object_gfx_list::ObjectGfxList,
        tilesets::TILESETS_COUNT,
    },
    snes_utils::addr::AddrSnes,
    RegionCode,
    RomInternalHeader,
};

pub mod color;
pub mod gfx_file;
pub mod palette;

// -------------------------------------------------------------------------------------------------

#[derive(Debug, Error)]
#[error("Cannot get GFX tile at WRAM address ${0:X}")]
pub struct TileFromWramError(AddrSnes);

// -------------------------------------------------------------------------------------------------

pub enum BlockGfx<'t> {
    Animated(Vec<[&'t Tile; 4]>),
    Static([&'t Tile; 4]),
}

pub struct Gfx {
    pub files:              Vec<GfxFile>,
    pub color_palettes:     ColorPalettes,
    pub object_gfx_list:    ObjectGfxList,
    pub animated_tile_data: AnimatedTileData,
}

// -------------------------------------------------------------------------------------------------

impl Gfx {
    pub fn parse(
        disasm: &mut RomDisassembly, levels: &[Level], internal_header: &RomInternalHeader,
    ) -> anyhow::Result<Self> {
        let revised_gfx =
            matches!(internal_header.region_code, RegionCode::Japan) || internal_header.version_number > 0;

        let mut files = Vec::with_capacity(GFX_FILES_META.len());
        for file_num in 0..GFX_FILES_META.len() {
            let file = GfxFile::new(disasm, file_num, revised_gfx)?;
            files.push(file);
        }

        Ok(Self {
            files,
            color_palettes: ColorPalettes::parse(disasm, levels)?,
            object_gfx_list: ObjectGfxList::parse(disasm)?,
            animated_tile_data: AnimatedTileData::parse(disasm)?,
        })
    }

    #[allow(clippy::erasing_op)]
    pub fn tiles_from_block(
        &self, block: &Block, tileset: usize, blue_pswitch: bool, silver_pswitch: bool, on_off_switch: bool,
        offset: u16,
    ) -> BlockGfx {
        assert!(tileset < TILESETS_COUNT);

        const BLANK_ANIM: [AddrSnes; 4] =
            [AddrSnes(0x7EAC20), AddrSnes(0x7EAC20), AddrSnes(0x7EAC20), AddrSnes(0x7EAC20)];

        match self.animated_tile_data.get_animation_frames_for_block(
            block,
            tileset,
            blue_pswitch,
            silver_pswitch,
            on_off_switch,
            offset,
        ) {
            Some(BLANK_ANIM) | None => {
                let ref_gfx = |tile| {
                    let file_num = self.object_gfx_list.gfx_file_for_object_tile(tile, tileset);
                    let tile_num = tile.tile_number() as usize % 0x80;
                    &self.files[file_num].tiles[tile_num]
                };
                BlockGfx::Static([
                    ref_gfx(block.upper_left),
                    ref_gfx(block.lower_left),
                    ref_gfx(block.upper_right),
                    ref_gfx(block.lower_right),
                ])
            }
            Some(frame_addrs) => {
                let frames = frame_addrs
                    .map(|frame_addr| {
                        let first_tile_vramadrr = block.upper_left.tile_vram_addr(offset);
                        let upper_left_offset = block.upper_left.tile_vram_addr(offset) - first_tile_vramadrr;
                        let lower_left_offset = block.lower_left.tile_vram_addr(offset) - first_tile_vramadrr;
                        let upper_right_offset = block.upper_right.tile_vram_addr(offset) - first_tile_vramadrr;
                        let lower_right_offset = block.lower_right.tile_vram_addr(offset) - first_tile_vramadrr;
                        [
                            self.tile_from_wram(frame_addr + upper_left_offset).unwrap(),
                            self.tile_from_wram(frame_addr + lower_left_offset).unwrap(),
                            self.tile_from_wram(frame_addr + upper_right_offset).unwrap(),
                            self.tile_from_wram(frame_addr + lower_right_offset).unwrap(),
                        ]
                    })
                    .to_vec();
                BlockGfx::Animated(frames)
            }
        }
    }

    pub fn tile_from_wram(&self, wram_addr: AddrSnes) -> Result<&Tile, TileFromWramError> {
        let (file, offset) = match wram_addr {
            // Mario graphics & berry animation
            AddrSnes(addr @ 0x7E2000..=0x7E7CFF) => (&self.files[0x32], addr - 0x7E2000),
            // Yoshi graphics & animated tiles
            AddrSnes(addr @ 0x7E7D00..=0x7EACFF) => (&self.files[0x33], addr - 0x7E7D00),
            // Unknown
            AddrSnes(_) => return Err(TileFromWramError(wram_addr)),
        };
        let index = offset as usize / (4 * 8);
        Ok(&file.tiles[index])
    }
}
