use crate::addr::AddressPC;
use std::{
    error::Error,
    fmt,
    io::Error as IoError,
};

use polyerror::create_error;

// Types -------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct RomAddressError {
    pub address: AddressPC,
}

#[derive(Debug)]
pub enum RomHeaderError {
    Checksum,
    DestinationCode(u8),
    MapMode(u8),
    RomType(u8),
}

#[derive(Debug)]
pub struct RomSizeError {
    pub size: usize,
}

create_error!(pub RomHeaderParseError: RomAddressError, RomHeaderError);
create_error!(pub RomParseError: RomAddressError, RomHeaderParseError, RomSizeError);
create_error!(pub RomReadError: IoError, RomParseError);

// Implementations ---------------------------------------------------------------------------------

impl fmt::Display for RomAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ROM doesn't contain address {}", self.address)
    }
}

impl fmt::Display for RomHeaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RomHeaderError::*;
        write!(f, "Could not load header: invalid {}.", match self {
            Checksum => String::from("checksum"),
            DestinationCode(dc) => format!("destination code {:#x}", dc),
            MapMode(mm) => format!("map mode {:#x}", mm),
            RomType(rt) => format!("ROM type {:#x}", rt),
        })
    }
}

impl fmt::Display for RomSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid ROM size: {}", self.size)
    }
}

impl Error for RomAddressError {}
impl Error for RomHeaderError {}
impl Error for RomSizeError {}