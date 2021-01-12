use crate::{
    addr::{AddrPc, AddrSnes},
    internal_header::MapMode,
};

use nom::{
    Err as NomErr,
    error::{
        Error as NomError,
        ErrorKind,
    },
};
use polyerror::create_error;
use std::{
    error::Error,
    fmt,
    io::Error as IoError,
};

// -------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub enum RomParseError {
    BadAddress(usize),
    BadSize(usize),
    InternalHeader,
    Level(usize),
    PaletteGlobal,
    PaletteLevel(usize),
}

#[derive(Debug)]
pub enum AddressConversionError {
    PcToSnes(AddrPc),
    SnesToPc(AddrSnes, MapMode),
}

create_error!(pub RomReadError: IoError, RomParseError);

// -------------------------------------------------------------------------------------------------

impl fmt::Display for RomParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RomParseError::*;
        write!(f, "{}", match self {
            BadAddress(addr) => format!("ROM doesn't contain PC address {}", addr),
            BadSize(size) => format!("Invalid ROM size: {}", size),
            InternalHeader => String::from("Parsing internal header failed"),
            Level(level_num) => format!("Invalid level: {:#x}", level_num),
            PaletteGlobal => String::from("Could not parse global level color palette"),
            PaletteLevel(level_num) => format!("Invalid level color palette: {:#x}", level_num),
        })
    }
}

impl Error for RomParseError {}

impl fmt::Display for AddressConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AddressConversionError::*;
        write!(f, "{}", match self {
            PcToSnes(addr) => format!("PC address {:#x} is too big for LoROM.", addr),
            SnesToPc(addr, map_mode) =>
                format!("Invalid SNES {} address: ${:x}", map_mode, addr),
        })
    }
}

impl Error for AddressConversionError {}

// -------------------------------------------------------------------------------------------------

pub fn nom_error(input: &[u8], kind: ErrorKind) -> NomErr<NomError<&[u8]>> {
    NomErr::Error(NomError::new(input, kind))
}