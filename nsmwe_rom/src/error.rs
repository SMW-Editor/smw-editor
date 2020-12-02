use crate::addr::AddressPc;
use std::{
    error::Error,
    fmt,
    io::Error as IoError,
};

use polyerror::create_error;

// Types -------------------------------------------------------------------------------------------

#[derive(Debug)]
pub enum RomParseError {
    BadAddress(AddressPc),
    BadSize(usize),
    InternalHeader,
}

create_error!(pub RomReadError: IoError, RomParseError);

// Implementations ---------------------------------------------------------------------------------

impl fmt::Display for RomParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RomParseError::*;
        write!(f, "{}", match self {
            BadAddress(addr) => format!("ROM doesn't contain address {}", addr),
            BadSize(size) => format!("Invalid ROM size: {}", size),
            InternalHeader => String::from("Parsing internal header failed"),
        })
    }
}

impl Error for RomParseError {}