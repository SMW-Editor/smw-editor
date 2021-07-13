use nom::{
    error::{Error as NomError, ErrorKind},
    Err as NomErr,
};
use thiserror::Error;

use crate::snes_utils::addr::{AddrPc, AddrSnes};

// -------------------------------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum AddressError {
    #[error("Invalid PC LoROM address {0:#x}")]
    InvalidPcLoRom(AddrPc),
    #[error("Invalid PC LoROM address {0:#x}")]
    InvalidPcHiRom(AddrPc),
    #[error("Invalid SNES LoROM address {0:#x}")]
    InvalidSnesLoRom(AddrSnes),
    #[error("Invalid SNES LoROM address {0:#x}")]
    InvalidSnesHiRom(AddrSnes),
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
pub enum ColorPaletteError {
    #[error("Failed to construct a level's back area color.")]
    LvBackAreaColor,
    #[error("Failed to construct a level's background palette.")]
    LvBackground,
    #[error("Failed to construct a level's foreground palette.")]
    LvForeground,
    #[error("Failed to construct a level's sprite palette.")]
    LvSprite,
    #[error("Failed to construct an overworld submap's layer 2 palette.")]
    OwLayer2,
}

#[derive(Debug, Error)]
pub enum InternalHeaderParseError {
    #[error("Couldn't find internal ROM header")]
    NotFound,
    #[error("Isolating Internal ROM Header")]
    IsolatingData,

    #[error("Reading checksum and complement at LoROM location")]
    ReadLoRomChecksum,
    #[error("Reading checksum and complement at HiROM location")]
    ReadHiRomChecksum,
    #[error("Reading Internal ROM Name")]
    ReadRomName,
    #[error("Reading Map Mode")]
    ReadMapMode,
    #[error("Reading ROM Type")]
    ReadRomType,
    #[error("Reading ROM Size")]
    ReadRomSize,
    #[error("Reading SRAM Size")]
    ReadSramSize,
    #[error("Reading Region Code")]
    ReadRegionCode,
    #[error("Reading Developer ID")]
    ReadDeveloperId,
    #[error("Reading Version Number")]
    ReadVersionNumber,
}

#[derive(Debug, Error)]
pub enum ColorPaletteParseError {
    #[error("Player Color Palette")]
    PlayerPalette,
    #[error("Overworld Layer1 Color Palette")]
    OverworldLayer1Palette,
    #[error("Overworld Layer2 Color Palette")]
    OverworldLayer3Palette,
    #[error("Overworld Sprite Color Palette")]
    OverworldSpritePalette,
    #[error("Overworld Submap {0}'s Normal Layer2 Color Palette")]
    OverworldLayer2NormalPalette(usize),
    #[error("Overworld Submap {0}'s Special Layer2 Color Palette")]
    OverworldLayer2SpecialPalette(usize),
    #[error("Overworld Layer2's Indirect Indices Table (${0:X})")]
    OverworldLayer2IndicesIndirect1Read(AddrSnes),
    #[error("Overworld Layer2's Index (${0:X})")]
    OverworldLayer2IndexRead(usize),

    #[error("Level Misc. Color Palette")]
    LevelMiscPalette,
    #[error("Level Layer3 Color Palette")]
    LevelLayer3Palette,
    #[error("Level Berry Color Palette")]
    LevelBerryPalette,
    #[error("Level Animated Color")]
    LevelAnimatedColor,
    #[error("Level {0:X}'s Back Area Color")]
    LevelBackAreaColor(usize),
    #[error("Level {0:X}'s Background Color Palette")]
    LevelBackgroundPalette(usize),
    #[error("Level {0:X}'s Foreground Color Palette")]
    LevelForegroundPalette(usize),
    #[error("Level {0:X}'s Sprite Color Palette")]
    LevelSpritePalette(usize),
}

#[derive(Debug, Error)]
pub enum GfxFileParseError {
    #[error("Address conversion: {0}")]
    AddressConversion(AddressError),
    #[error("Isolating data")]
    IsolatingData,
    #[error("Decompressing data: {0}")]
    DecompressingData(DecompressionError),
    #[error("Parsing tile")]
    ParsingTile,
}

#[derive(Debug, Error)]
pub enum SecondaryEntranceParseError {
    #[error("Converting SNES address of Secondary Entrance Tables to PC")]
    TablesAddressConversion,
    #[error("Reading Secondary Entrance data")]
    Read,
}

#[derive(Debug, Error)]
pub enum SecondaryHeaderParseError {
    #[error("Converting SNES address of Secondary Header Tables to PC")]
    AddressConversion(AddressError),
    #[error("Reading Secondary Header data")]
    Read,
}

#[derive(Debug, Error)]
pub enum LevelParseError {
    #[error("Converting SNES address of Layer1 data into PC")]
    Layer1AddressConversion,
    #[error("Converting SNES address of Layer2 data to PC")]
    Layer2AddressConversion,
    #[error("Converting SNES address of Layer2 pointer to PC")]
    Layer2PtrAddressConversion,
    #[error("Converting SNES address of primary header into PC")]
    PrimaryHeaderAddressConversion,
    #[error("Converting SNES address of address of Sprite data to PC")]
    SpritePtrAddressConversion,
    #[error("Converting SNES address of sprite header to PC")]
    SpriteAddressConversion,

    #[error("Reading address of Layer1")]
    Layer1AddressRead,
    #[error("Reading address of Layer2")]
    Layer2AddressRead,
    #[error("Reading address of Sprite data")]
    SpriteAddressRead,

    #[error("Isolating Layer1 data")]
    Layer1Isolate,
    #[error("Isolating Layer2 data")]
    Layer2Isolate,
    #[error("Isolating Sprite data")]
    SpriteIsolate,

    #[error("Reading Primary Header")]
    PrimaryHeaderRead,
    #[error("Reading Secondary Header")]
    SecondaryHeaderRead(SecondaryHeaderParseError),
    #[error("Reading Sprite Header")]
    SpriteHeaderRead,

    #[error("Reading Layer1 object data")]
    Layer1Read,
    #[error("Parsing Layer2 object data")]
    Layer2Read,
    #[error("Reading Layer2 background: {0}")]
    Layer2BackgroundRead(DecompressionError),
    #[error("Reading Sprite data")]
    SpriteRead,
}

#[derive(Debug, Error)]
pub enum RomParseError {
    #[error("ROM doesn't contain PC address {0:#x}")]
    BadAddress(usize),
    #[error("Invalid ROM size: {0} ({0:#x})")]
    BadSize(usize),
    #[error("Invalid GFX file {0:X}: {1}")]
    GfxFile(usize, GfxFileParseError),
    #[error("Parsing internal header failed")]
    InternalHeader(InternalHeaderParseError),
    #[error("File IO Error")]
    IoError,
    #[error("Failed to parse level {0:#X}: {1}")]
    Level(usize, LevelParseError),
    #[error("Failed to read secondary entrance {0:#X}: {1}")]
    SecondaryEntrance(usize, SecondaryEntranceParseError),
    #[error("Could not parse color palettes")]
    ColorPalettes(ColorPaletteParseError),
}

pub type ParseErr<'a> = nom::Err<nom::error::Error<&'a [u8]>>;

// -------------------------------------------------------------------------------------------------

pub fn nom_error(input: &[u8], kind: ErrorKind) -> NomErr<NomError<&[u8]>> {
    NomErr::Error(NomError::new(input, kind))
}
