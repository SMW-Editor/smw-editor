#![allow(clippy::identity_op)]

pub mod compression;
pub mod disassembler;
pub mod error;
pub mod graphics;
pub mod internal_header;
pub mod level;
pub mod objects;
pub mod snes_utils;

use std::{fs, path::Path};

pub use crate::internal_header::RomInternalHeader;
use crate::{
    disassembler::RomDisassembly,
    error::RomParseError,
    graphics::{
        gfx_file::{GfxFile, GFX_FILES_META},
        palette::ColorPalettes,
    },
    level::{
        secondary_entrance::{SecondaryEntrance, SECONDARY_ENTRANCE_TABLE},
        Level,
        LEVEL_COUNT,
    },
    objects::tilesets::Tilesets,
    snes_utils::rom::Rom,
};

pub struct SmwRom {
    pub disassembly:         RomDisassembly,
    pub internal_header:     RomInternalHeader,
    pub levels:              Vec<Level>,
    pub secondary_entrances: Vec<SecondaryEntrance>,
    pub color_palettes:      ColorPalettes,
    pub gfx_files:           Vec<GfxFile>,
    pub map16_tilesets:      Tilesets,
}

impl SmwRom {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, RomParseError> {
        log::info!("Reading ROM from file: {}", path.as_ref().display());
        let smw_rom = fs::read(path)
            .map_err(|err| {
                log::error!("Could not read ROM: {}", err);
                RomParseError::IoError
            })
            .and_then(|rom_data| Rom::new(rom_data).map_err(RomParseError::BadRom))
            .and_then(Self::from_rom);

        if smw_rom.is_ok() {
            log::info!("Success parsing ROM");
        }

        smw_rom
    }

    pub fn from_rom(rom: Rom) -> Result<Self, RomParseError> {
        log::info!("Parsing internal ROM header");
        let internal_header = RomInternalHeader::parse(&rom).map_err(RomParseError::InternalHeader)?;

        log::info!("Parsing level data");
        let levels = Self::parse_levels(&rom)?;

        log::info!("Parsing secondary entrances");
        let secondary_entrances = Self::parse_secondary_entrances(&rom)?;

        log::info!("Parsing color palettes");
        let color_palettes = ColorPalettes::parse(&rom, &levels).map_err(RomParseError::ColorPalettes)?;

        log::info!("Parsing GFX files");
        let gfx_files = Self::parse_gfx_files(&rom)?;

        log::info!("Parsing Map16 tilesets");
        let map16_tilesets = Tilesets::parse(&rom).map_err(RomParseError::Map16Tilesets)?;

        log::info!("Creating disassembly map");
        let disassembly = RomDisassembly::new(&rom, &internal_header);

        Ok(Self {
            disassembly,
            internal_header,
            levels,
            secondary_entrances,
            color_palettes,
            gfx_files,
            map16_tilesets,
        })
    }

    fn parse_levels(rom: &Rom) -> Result<Vec<Level>, RomParseError> {
        let mut levels = Vec::with_capacity(LEVEL_COUNT);
        for level_num in 0..LEVEL_COUNT {
            let level = Level::parse(rom, level_num).map_err(|e| RomParseError::Level(level_num, e))?;
            levels.push(level);
        }
        Ok(levels)
    }

    fn parse_secondary_entrances(rom: &Rom) -> Result<Vec<SecondaryEntrance>, RomParseError> {
        let mut secondary_entrances = Vec::with_capacity(SECONDARY_ENTRANCE_TABLE.size);
        for entrance_id in 0..SECONDARY_ENTRANCE_TABLE.size {
            let entrance = SecondaryEntrance::read_from_rom(rom, entrance_id)
                .map_err(|e| RomParseError::SecondaryEntrance(entrance_id, e))?;
            secondary_entrances.push(entrance);
        }
        Ok(secondary_entrances)
    }

    fn parse_gfx_files(rom: &Rom) -> Result<Vec<GfxFile>, RomParseError> {
        let mut gfx_files = Vec::with_capacity(GFX_FILES_META.len());
        for file_num in 0..GFX_FILES_META.len() {
            let file = GfxFile::new(rom, file_num).map_err(|e| RomParseError::GfxFile(file_num, e))?;
            gfx_files.push(file);
        }
        Ok(gfx_files)
    }
}
