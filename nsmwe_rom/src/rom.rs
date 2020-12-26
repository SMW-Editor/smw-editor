pub use self::constants::*;

use crate::{
    error::{
        RomParseError,
        RomReadError,
    },
    internal_header::RomInternalHeader,
    level::level::Level,
};

use std::{
    fs,
    path::Path,
};

pub mod constants {
    pub const SMC_HEADER_SIZE: usize = 0x200;
}

pub struct Rom {
    pub internal_header: RomInternalHeader,
    pub levels: Vec<Level>,
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

    pub fn from_raw(rom_data: &[u8]) -> Result<Rom, RomParseError> {
        let rom_data = Rom::trim_smc_header(rom_data)?;

        let internal_header =
            match RomInternalHeader::from_rom_data(rom_data) {
                Ok((_, header)) => header,
                Err(_) => return Err(RomParseError::InternalHeader),
            };

        Ok(Rom {
            internal_header,
            levels: Vec::new(),
        })
    }

    pub fn trim_smc_header(rom_data: &[u8]) -> Result<&[u8], RomParseError> {
        let size = rom_data.len() % 0x400;
        if size == SMC_HEADER_SIZE {
            Ok(&rom_data[SMC_HEADER_SIZE..])
        } else if size == 0 {
            Ok(&rom_data[..])
        } else {
            Err(RomParseError::BadSize(size))
        }
    }
}
