use std::{fs, path::Path};

pub use self::constants::*;
use crate::{
    error::RomParseError,
    graphics::{
        gfx_file::{GfxFile, GFX_FILES_META},
        palette::ColorPalettes,
    },
    internal_header::RomInternalHeader,
    level::{Level, LEVEL_COUNT},
};

pub mod constants {
    pub const SMC_HEADER_SIZE: usize = 0x200;
}

type RpResult<T> = Result<T, RomParseError>;

pub struct Rom {
    pub internal_header: RomInternalHeader,
    pub levels:          Vec<Level>,
    pub color_palettes:  ColorPalettes,
    pub gfx_files:       Vec<GfxFile>,
}

impl Rom {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, RomParseError> {
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
                log::error!("Couldn't read ROM: {}", err);
                Err(RomParseError::IoError)
            }
        }
    }

    pub fn from_raw(rom_data: &[u8]) -> RpResult<Self> {
        let rom_data = Self::trim_smc_header(rom_data)?;

        log::info!("Parsing internal ROM header");
        let internal_header = Self::parse_internal_header(rom_data)?;

        log::info!("Parsing level data");
        let levels = Self::parse_levels(rom_data)?;

        log::info!("Parsing color palettes");
        let color_palettes = ColorPalettes::parse(rom_data, &levels)?;

        log::info!("Parsing GFX files");
        let gfx_files = Self::parse_gfx_files(rom_data)?;

        Ok(Self { internal_header, levels, color_palettes, gfx_files })
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

    fn parse_internal_header(rom_data: &[u8]) -> RpResult<RomInternalHeader> {
        match RomInternalHeader::parse(rom_data) {
            Ok((_, header)) => Ok(header),
            Err(_) => Err(RomParseError::InternalHeader),
        }
    }

    fn parse_levels(rom_data: &[u8]) -> RpResult<Vec<Level>> {
        let mut levels = Vec::with_capacity(LEVEL_COUNT);
        for level_num in 0..LEVEL_COUNT {
            match Level::parse(rom_data, level_num) {
                Ok((_, level)) => levels.push(level),
                Err(_) => return Err(RomParseError::Level(level_num)),
            }
        }
        Ok(levels)
    }

    fn parse_gfx_files(rom_data: &[u8]) -> RpResult<Vec<GfxFile>> {
        let mut gfx_files = Vec::with_capacity(GFX_FILES_META.len());
        for (i, &(tile_format, addr, size_bytes)) in GFX_FILES_META.iter().enumerate() {
            match GfxFile::new(rom_data, tile_format, addr, size_bytes) {
                Ok((_, file)) => gfx_files.push(file),
                Err(_) => return Err(RomParseError::GfxFile(tile_format, i, size_bytes)),
            }
        }
        Ok(gfx_files)
    }
}
