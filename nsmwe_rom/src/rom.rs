pub use self::constants::*;

use crate::{
    error::RomParseError,
    graphics::{
        gfx_file::{
            GfxFile,
            GFX_FILES_META,
        },
        palette::{
            GlobalLevelColorPalette,
            LevelColorPaletteSet,
        },
    },
    internal_header::RomInternalHeader,
    level::{
        Level,
        LEVEL_COUNT,
    },
};

use std::{
    fs,
    path::Path,
    rc::Rc,
};

pub mod constants {
    pub const SMC_HEADER_SIZE: usize = 0x200;
}

type RpResult<T> = Result<T, RomParseError>;

pub struct Rom {
    pub internal_header: RomInternalHeader,
    pub levels: Vec<Level>,
    pub global_level_color_palette: Rc<GlobalLevelColorPalette>,
    pub level_color_palette_set: LevelColorPaletteSet,
    pub gfx_files: Vec<GfxFile>,
}

impl Rom {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Rom, RomParseError> {
        match fs::read(path) {
            Ok(rom_data) => match Rom::from_raw(&rom_data) {
                Ok(rom) => Ok(rom),
                Err(err) => Err(err),
            },
            Err(_) => Err(RomParseError::IoError),
        }
    }

    pub fn from_raw(rom_data: &[u8]) -> RpResult<Rom> {
        let rom_data = Rom::trim_smc_header(rom_data)?;

        let internal_header = Rom::get_internal_header(rom_data)?;
        let levels = Rom::get_levels(rom_data)?;
        let global_level_color_palette = Rom::get_global_level_color_palette(rom_data)?;
        let level_color_palette_set = LevelColorPaletteSet::parse(rom_data, &levels)?;
        let gfx_files = Rom::get_gfx_files(rom_data)?;

        Ok(Rom {
            internal_header,
            levels,
            global_level_color_palette,
            level_color_palette_set,
            gfx_files,
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
            Err(_) => Err(RomParseError::InternalHeader),
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

    fn get_global_level_color_palette(rom_data: &[u8]) -> RpResult<Rc<GlobalLevelColorPalette>> {
        match GlobalLevelColorPalette::parse(rom_data) {
            Ok((_, palette)) => Ok(Rc::new(palette)),
            Err(_) => Err(RomParseError::PaletteGlobal),
        }
    }

    fn get_gfx_files(rom_data: &[u8]) -> RpResult<Vec<GfxFile>> {
        let mut gfx_files = Vec::with_capacity(GFX_FILES_META.len());
        for &(tile_format, addr, size_bytes) in GFX_FILES_META.iter() {
            match GfxFile::new(rom_data, tile_format, addr, size_bytes) {
                Ok((_, file)) => gfx_files.push(file),
                Err(_) => return Err(RomParseError::GfxFile(tile_format, addr, size_bytes)),
            }
        }
        Ok(gfx_files)
    }
}
