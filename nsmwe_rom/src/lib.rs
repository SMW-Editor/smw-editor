pub mod addr;
pub mod graphics;
pub mod internal_header;

pub use crate::internal_header::RomInternalHeader;

pub use self::{
    constants::*,
    helpers::*,
};

use std::path::Path;

pub struct Rom {
    pub internal_header: RomInternalHeader,
}

impl Rom {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Rom, String> {
        match std::fs::read(path) {
            Ok(rom_data) => Rom::from_raw(&rom_data),
            Err(err) => Err(format!("Unable to open ROM file: {}", err.to_string())),
        }
    }

    pub fn from_raw(data: &[u8]) -> Result<Rom, String> {
        let smc_header_offset = if has_smc_header(data)? { SMC_HEADER_SIZE } else { 0 };
        let internal_header = RomInternalHeader::from_rom_data(data, smc_header_offset)?;

        Ok(Rom {
            internal_header,
        })
    }
}

mod constants {
    pub const SMC_HEADER_SIZE: u32 = 0x200;
}

mod helpers {
    pub fn has_smc_header(data: &[u8]) -> Result<bool, String> {
        use crate::SMC_HEADER_SIZE;

        let rem = (data.len() % 0x400) as u32;
        if rem == SMC_HEADER_SIZE {
            Ok(true)
        } else if rem == 0 {
            Ok(false)
        } else {
            Err(format!("Invalid header size: {}.", rem))
        }
    }

    pub fn get_byte_at(data: &[u8], idx: usize) -> Result<u8, String> {
        if let Some(b) = data.get(idx as usize) {
            Ok(*b)
        } else {
            Err(String::from("Could not locate header: ROM size is too small."))
        }
    }

    pub fn get_word_at(data: &[u8], idx: usize) -> Result<u16, String> {
        use std::convert::TryInto;

        if let Some(slice) = data.get(idx..=idx + 1) {
            Ok(u16::from_le_bytes(slice.try_into().unwrap()))
        } else {
            Err(String::from("Could not locate header: ROM size is too small."))
        }
    }

    pub fn get_slice_at(data: &[u8], idx: usize, size: usize) -> Result<&[u8], String> {
        let idx = idx as usize;
        if let Some(slice) = data.get(idx..idx + size) {
            Ok(slice)
        } else {
            Err(String::from("Could not locate header: ROM size is too small."))
        }
    }
}