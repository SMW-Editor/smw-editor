pub use constants::*;

use crate::{
    error::{
        RomParseError,
        RomReadError,
    },
    internal_header::RomInternalHeader,
};
use self::{
    helpers::*,
};

use std::{
    fs,
    path::Path,
};

pub struct Rom {
    pub internal_header: RomInternalHeader,
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

    pub fn from_raw(data: &[u8]) -> Result<Rom, RomParseError> {
        let smc_header_offset = if data_has_smc_header(data)? { SMC_HEADER_SIZE } else { 0 };
        let (_input, internal_header) =
            match RomInternalHeader::from_rom_data(data, smc_header_offset) {
                Ok(res) => res,
                Err(_) => return Err(RomParseError::InternalHeader),
            };

        Ok(Rom {
            internal_header,
        })
    }
}

pub mod constants {
    pub const SMC_HEADER_SIZE: usize = 0x200;
}

mod helpers {
    use crate::error::RomParseError;

    pub fn data_has_smc_header(data: &[u8]) -> Result<bool, RomParseError> {
        use crate::SMC_HEADER_SIZE;

        let size = data.len() % 0x400;
        if size == SMC_HEADER_SIZE {
            Ok(true)
        } else if size == 0 {
            Ok(false)
        } else {
            Err(RomParseError::BadSize(size))
        }
    }
}