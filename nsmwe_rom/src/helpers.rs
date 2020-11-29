use crate::{
    addr::AddressPC,
    error::RomAddressError,
};

use bytemuck::try_from_bytes;

pub fn get_byte_at(data: &[u8], idx: usize) -> Result<u8, RomAddressError> {
    if let Some(b) = data.get(idx as usize) {
        Ok(*b)
    } else {
        Err(RomAddressError { address: idx as AddressPC })
    }
}

pub fn get_word_at(data: &[u8], idx: usize) -> Result<u16, RomAddressError> {
    if let Some(slice) = data.get(idx..=idx + 1) {
        Ok(try_from_bytes::<u16>(slice).unwrap().to_be())
    } else {
        Err(RomAddressError { address: idx as AddressPC })
    }
}

pub fn get_slice_at(data: &[u8], idx: usize, size: usize) -> Result<&[u8], RomAddressError> {
    let idx = idx as usize;
    if let Some(slice) = data.get(idx..idx + size) {
        Ok(slice)
    } else {
        Err(RomAddressError { address: idx as AddressPC })
    }
}