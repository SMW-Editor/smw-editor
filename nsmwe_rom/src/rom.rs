pub use self::constants::*;

use crate::{
    error::{RomParseError, RomReadError},
    graphics::{
        palette::ColorPalette,
        pointer_tables::LEVEL_PALETTES,
    },
    internal_header::RomInternalHeader,
    level::{
        level::Level,
        LEVEL_COUNT,
    },
};

use std::{
    fs,
    path::Path,
};

pub mod constants {
    pub const SMC_HEADER_SIZE: usize = 0x200;
}

type RpResult<T> = Result<T, RomParseError>;

pub struct Rom {
    pub internal_header: RomInternalHeader,
    pub levels: Vec<Level>,
    pub color_palettes: Vec<ColorPalette>,
}

impl Rom {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Rom, RomReadError> {
        match fs::read(path) {
            Ok(rom_data) => match Rom::from_raw(&rom_data) {
                Ok(rom) => Ok(rom),
                Err(err) => Err(err.into()),
            },
            Err(err) => Err(err.into()),
        }
    }

    pub fn from_raw(rom_data: &[u8]) -> RpResult<Rom> {
        let rom_data = Rom::trim_smc_header(rom_data)?;

        let internal_header = Rom::get_internal_header(rom_data)?;
        let levels = Rom::get_levels(rom_data)?;
        let color_palettes = Rom::get_color_palettes(rom_data, &levels)?;

        Ok(Rom {
            internal_header,
            levels,
            color_palettes,
        })
    }

    fn trim_smc_header(rom_data: &[u8]) -> RpResult<&[u8]> {
        let size = rom_data.len() % 0x400;
        if size == SMC_HEADER_SIZE {
            Ok(&rom_data[SMC_HEADER_SIZE..])
        } else if size == 0 {
            Ok(&rom_data[..])
        } else {
            Err(RomParseError::BadSize(size))
        }
    }

    fn get_internal_header(rom_data: &[u8]) -> RpResult<RomInternalHeader> {
        match RomInternalHeader::from_rom_data(rom_data) {
            Ok((_, header)) => Ok(header),
            Err(_) => return Err(RomParseError::InternalHeader),
        }
    }

    fn get_levels(rom_data: &[u8]) -> RpResult<Vec<Level>> {
        let mut levels = Vec::with_capacity(LEVEL_COUNT);
        for level_num in 0..LEVEL_COUNT {
            match Level::from_rom_data(rom_data, level_num) {
                Ok((_, level)) => levels.push(level),
                Err(_) => return Err(RomParseError::Level(level_num)),
            }
        }
        Ok(levels)
    }

    fn get_color_palettes(rom_data: &[u8], levels: &[Level]) -> RpResult<Vec<ColorPalette>> {
        let mut palettes = Vec::with_capacity(LEVEL_COUNT);
        for (level_num, level) in levels.iter().enumerate() {
            let palette_addr = LEVEL_PALETTES + (3 * level_num);
            match ColorPalette::parse_level_palette(rom_data, palette_addr, &level.primary_header) {
                Ok((_, palette)) => palettes.push(palette),
                Err(_) => return Err(RomParseError::PaletteLevel(level_num)),
            }
        }
        Ok(palettes)
    }
}
