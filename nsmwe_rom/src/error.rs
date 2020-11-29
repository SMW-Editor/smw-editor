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
pub struct RomHeaderChecksumError;

#[derive(Debug)]
pub enum RomHeaderDataError {
    DestinationCode,
    MapMode,
    RomType,
}

#[derive(Debug)]
pub struct RomSizeError {
    pub size: usize,
}

create_error!(pub RomHeaderError: RomAddressError, RomHeaderDataError, RomHeaderFindError);
create_error!(pub RomHeaderFindError: RomAddressError, RomHeaderChecksumError);
create_error!(pub RomParseError: RomAddressError, RomHeaderError, RomSizeError);
create_error!(pub RomReadError: IoError, RomParseError);

// Implementations ---------------------------------------------------------------------------------

impl fmt::Display for RomAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ROM doesn't contain address {}", self.address)
    }
}

impl fmt::Display for RomHeaderChecksumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cannot find internal header: both checksums are incorrect.")
    }
}

impl fmt::Display for RomHeaderDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RomHeaderDataError::*;
        write!(f, "Invalid {}.", match self {
            DestinationCode => "destination code",
            MapMode => "map mode",
            RomType => "ROM type",
        })
    }
}

impl fmt::Display for RomSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid ROM size: {}", self.size)
    }
}

impl Error for RomAddressError {}
impl Error for RomHeaderChecksumError {}
impl Error for RomHeaderDataError {}
impl Error for RomSizeError {}