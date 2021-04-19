use crate::{
    addr::{AddrPc, AddrSnes},
    graphics::gfx_file::TileFormat,
    internal_header::MapMode,
};

use nom::{
    Err as NomErr,
    error::{
        Error as NomError,
        ErrorKind,
    },
};

use thiserror::Error;

// -------------------------------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum AddressConversionError {
    #[error("PC address {0:#x} is too big for LoROM.")]
    PcToSnes(AddrPc),
    #[error("Invalid SNES {1} address: ${0:x}")]
    SnesToPc(AddrSnes, MapMode),
}

#[derive(Debug, Error)]
#[error("Failed to decompress data: {0}")]
pub struct DecompressionError(pub &'static str);

#[derive(Debug, Error)]
pub enum GfxTileError {
    #[error("Failed to convert an indexed tile to Abgr1555")]
    ToAbgr1555,
    #[error("Failed to convert an indexed tile to Rgba32")]
    ToRgba32,
}

#[derive(Debug, Error)]
pub enum LevelPaletteError {
    #[error("Failed to construct a level's back area color.")]
    BackAreaColor,
    #[error("Failed to construct a level's background palette.")]
    Background,
    #[error("Failed to construct a level's foreground palette.")]
    Foreground,
    #[error("Failed to construct a level's sprite palette.")]
    Sprite,
}

#[derive(Debug, Error)]
pub enum RomParseError {
    #[error("ROM doesn't contain PC address {0:#x}")]
    BadAddress(usize),
    #[error("Invalid ROM size: {0} ({0:#x})")]
    BadSize(usize),
    #[error("Invalid GFX file - tile format: {0}, file num: {1:X}, size: {2}B")]
    GfxFile(TileFormat, usize, usize),
    #[error("Parsing internal header failed")]
    InternalHeader,
    #[error("File IO Error")]
    IoError,
    #[error("Invalid level: {0:#X}")]
    Level(usize),
    #[error("Could not parse global level color palette")]
    PaletteGlobal,
    #[error("Invalid color palette in level {0:#X}")]
    PaletteSetLevel(usize),
}

// -------------------------------------------------------------------------------------------------

pub fn nom_error(input: &[u8], kind: ErrorKind) -> NomErr<NomError<&[u8]>> {
    NomErr::Error(NomError::new(input, kind))
}
