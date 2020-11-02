pub mod addr;
pub mod internal_header;

pub use constants::*;
pub use helpers::*;

use internal_header::InternalHeader;

pub struct Rom {
    internal_header: InternalHeader,
}

pub fn parse_rom_data(data: &[u8]) -> Result<Rom, String> {
    let smc_header_offset = if has_smc_header(data)? { SMC_HEADER_SIZE } else { 0 };
    let internal_header = InternalHeader::from_rom_data(data, smc_header_offset)?;

    Ok(Rom {
        internal_header,
    })
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

    pub fn get_byte_at(data: &[u8], idx: u32) -> Result<u8, String> {
        if let Some(b) = data.get(idx as usize) {
            Ok(*b)
        } else {
            Err(String::from("Could not locate header: ROM size is too small."))
        }
    }

    pub fn get_word_at(data: &[u8], idx: u32) -> Result<u16, String> {
        use std::convert::TryInto;

        let idx = idx as usize;
        if let Some(slice) = data.get(idx..=idx + 1) {
            Ok(u16::from_le_bytes(slice.try_into().unwrap()))
        } else {
            Err(String::from("Could not locate header: ROM size is too small."))
        }
    }

    pub fn get_slice_at(data: &[u8], idx: u32, size: usize) -> Result<&[u8], String> {
        let idx = idx as usize;
        if let Some(slice) = data.get(idx..idx + size) {
            Ok(slice)
        } else {
            Err(String::from("Could not locate header: ROM size is too small."))
        }
    }
}