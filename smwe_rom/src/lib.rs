#![allow(clippy::identity_op)]

use std::{fs, path::Path};

pub use crate::{constants::*, internal_header::RomInternalHeader};
use crate::{
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
};

pub mod compression;
pub mod error;
pub mod graphics;
pub mod internal_header;
pub mod level;
pub mod snes_utils;

pub mod constants {
    pub const SMC_HEADER_SIZE: usize = 0x200;
}

type RpResult<T> = Result<T, RomParseError>;

pub struct SmwRom {
    pub internal_header:     RomInternalHeader,
    pub levels:              Vec<Level>,
    pub secondary_entrances: Vec<SecondaryEntrance>,
    pub color_palettes:      ColorPalettes,
    pub gfx_files:           Vec<GfxFile>,
}

impl SmwRom {
    pub fn from_file<P: AsRef<Path>>(path: P) -> RpResult<Self> {
        log::info!("Reading ROM from file: {}", path.as_ref().display());
        match fs::read(path) {
            Ok(rom_data) => match Self::from_raw(&rom_data) {
                Ok(rom) => {
                    log::info!("Success parsing ROM");
                    Ok(rom)
                }
                Err(err) => {
                    log::error!("Failed to parse ROM: {}", err);
                    Err(err)
                }
            },
            Err(err) => {
                log::error!("Could not read ROM: {}", err);
                Err(RomParseError::IoError)
            }
        }
    }

    pub fn from_raw(rom_data: &[u8]) -> RpResult<Self> {
        let rom_data = Self::trim_smc_header(rom_data)?;

        log::info!("Parsing internal ROM header");
        let internal_header = RomInternalHeader::parse(rom_data).map_err(RomParseError::InternalHeader)?;

        log::info!("Parsing level data");
        let levels = Self::parse_levels(rom_data)?;

        log::info!("Parsing secondary entrances");
        let secondary_entrances = Self::parse_secondary_entrances(rom_data)?;

        log::info!("Parsing color palettes");
        let color_palettes = ColorPalettes::parse(rom_data, &levels).map_err(RomParseError::ColorPalettes)?;

        log::info!("Parsing GFX files");
        let gfx_files = Self::parse_gfx_files(rom_data)?;

        Ok(Self { internal_header, levels, secondary_entrances, color_palettes, gfx_files })
    }

    fn trim_smc_header(rom_data: &[u8]) -> RpResult<&[u8]> {
        let size = rom_data.len() % 0x400;
        if size == SMC_HEADER_SIZE {
            Ok(&rom_data[SMC_HEADER_SIZE..])
        } else if size == 0 {
            Ok(&rom_data)
        } else {
            Err(RomParseError::BadSize(size))
        }
    }

    fn parse_levels(rom_data: &[u8]) -> RpResult<Vec<Level>> {
        let mut levels = Vec::with_capacity(LEVEL_COUNT);
        for level_num in 0..LEVEL_COUNT {
            let level = Level::parse(rom_data, level_num).map_err(|e| RomParseError::Level(level_num, e))?;
            levels.push(level);
        }
        Ok(levels)
    }

    fn parse_secondary_entrances(rom_data: &[u8]) -> RpResult<Vec<SecondaryEntrance>> {
        let mut secondary_entrances = Vec::with_capacity(SECONDARY_ENTRANCE_TABLE.size);
        for entrance_id in 0..SECONDARY_ENTRANCE_TABLE.size {
            let entrance = SecondaryEntrance::read_from_rom(rom_data, entrance_id)
                .map_err(|e| RomParseError::SecondaryEntrance(entrance_id, e))?;
            secondary_entrances.push(entrance);
        }
        Ok(secondary_entrances)
    }

    fn parse_gfx_files(rom_data: &[u8]) -> RpResult<Vec<GfxFile>> {
        let mut gfx_files = Vec::with_capacity(GFX_FILES_META.len());
        for file_num in 0..GFX_FILES_META.len() {
            let file = GfxFile::new(rom_data, file_num).map_err(|e| RomParseError::GfxFile(file_num, e))?;
            gfx_files.push(file);
        }
        Ok(gfx_files)
    }
}
