use crate::{
    constants::SMC_HEADER_SIZE,
    error::{
        RomParseError,
        RomReadError,
    },
    internal_header::RomInternalHeader,
};
use self::helpers::*;

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
        let internal_header = RomInternalHeader::from_rom_data(data, smc_header_offset)?;

        Ok(Rom {
            internal_header,
        })
    }
}

mod helpers {
    use crate::error::RomSizeError;

    pub fn data_has_smc_header(data: &[u8]) -> Result<bool, RomSizeError> {
        use crate::SMC_HEADER_SIZE;
        let rem = (data.len() % 0x400) as u32;
        if rem == SMC_HEADER_SIZE {
            Ok(true)
        } else if rem == 0 {
            Ok(false)
        } else {
            Err(RomSizeError { size: rem as usize })
        }
    }
}