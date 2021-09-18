use std::ops::Range;

use thiserror::Error;

use crate::snes_utils::{
    addr::{AddrPc, AddrSnes},
    rom_slice::{PcSlice, SnesSlice},
};

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
pub enum DecompressionError {
    #[error("Decompression with LC-RLE1:\n- {0}")]
    LcRle1(LcRle1Error),
    #[error("Decompression with LC-LZ2:\n- {0}")]
    LcLz2(LcLz2Error),
}

#[derive(Debug, Error)]
pub enum LcRle1Error {
    #[error("Wrong command: {0:03b}")]
    Command(u8),
    #[error("Direct Copy - Cannot read {0} bytes")]
    DirectCopy(usize),
    #[error("Byte Fill - Cannot read byte")]
    ByteFill,
}

#[derive(Debug, Error)]
pub enum LcLz2Error {
    #[error("Wrong command: {0:03b}")]
    Command(u8),
    #[error("Long Length - Wrong command: {0:03b}")]
    LongLengthCommand(u8),
    #[error("Long Length - Cannot read second byte of header")]
    LongLength,
    #[error("Direct Copy - Cannot read {0} bytes")]
    DirectCopy(usize),
    #[error("Byte Fill - Cannot read byte")]
    ByteFill,
    #[error("Word Fill - Cannot read word")]
    WordFill,
    #[error("Increasing Fill - Cannot read byte")]
    IncreasingFill,
    #[error("Repeat - Cannot read offset")]
    RepeatIncomplete,
    #[error("Repeat - Range ({}..{}) out of bounds (out buffer size: {1})", .0.start, .0.end)]
    RepeatRangeOutOfBounds(Range<usize>, usize),
    #[error("Double Long Length")]
    DoubleLongLength,
}

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
    #[error("Isolating Internal ROM Header:\n- {0}")]
    IsolatingData(RomError),

    #[error("Reading checksum and complement at LoROM location:\n- {0}")]
    ReadLoRomChecksum(RomError),
    #[error("Reading checksum and complement at HiROM location:\n- {0}")]
    ReadHiRomChecksum(RomError),
    #[error("Reading Internal ROM Name:\n- {0}")]
    ReadRomName(RomError),
    #[error("Reading Map Mode:\n- {0}")]
    ReadMapMode(RomError),
    #[error("Reading ROM Type:\n- {0}")]
    ReadRomType(RomError),
    #[error("Reading ROM Size:\n- {0}")]
    ReadRomSize(RomError),
    #[error("Reading SRAM Size:\n- {0}")]
    ReadSramSize(RomError),
    #[error("Reading Region Code:\n- {0}")]
    ReadRegionCode(RomError),
    #[error("Reading Developer ID:\n- {0}")]
    ReadDeveloperId(RomError),
    #[error("Reading Version Number:\n- {0}")]
    ReadVersionNumber(RomError),
}

#[derive(Copy, Clone, Debug, Error)]
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
    #[error("Overworld Layer2's Indirect Indices Table (${0})")]
    OverworldLayer2IndicesIndirect1Read(SnesSlice),
    #[error("Overworld Layer2's Index (${0})")]
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
    #[error("Isolating GFX data:\n- {0}")]
    IsolatingData(RomError),
    #[error("Decompressing GFX data:\n- {0}")]
    DecompressingData(LcLz2Error),
    #[error("Parsing GFX tile")]
    ParsingTile,
}

#[derive(Debug, Error)]
pub enum LevelParseError {
    #[error("Reading address of Layer1:\n- {0}")]
    Layer1AddressRead(RomError),
    #[error("Reading address of Layer2:\n- {0}")]
    Layer2AddressRead(RomError),
    #[error("Reading address of Sprite data:\n- {0}")]
    SpriteAddressRead(RomError),

    #[error("Isolating Layer2 data:\n- {0}")]
    Layer2Isolate(RomError),

    #[error("Reading Primary Header:\n- {0}")]
    PrimaryHeaderRead(RomError),
    #[error("Reading Secondary Header:\n- {0}")]
    SecondaryHeaderRead(RomError),
    #[error("Reading Sprite Header:\n- {0}")]
    SpriteHeaderRead(RomError),

    #[error("Reading Layer1 object data:\n- {0}")]
    Layer1Read(RomError),
    #[error("Parsing Layer2 object data:\n- {0}")]
    Layer2Read(RomError),
    #[error("Reading Layer2 background:\n- {0}")]
    Layer2BackgroundRead(DecompressionError),
    #[error("Reading Sprite data:\n- {0}")]
    SpriteRead(RomError),
}

#[derive(Debug, Error)]
pub enum RomError {
    #[error("Empty ROM file")]
    Empty,
    #[error("Invalid ROM size (not a multiple of 512 bytes): {0} ({0:#x})")]
    Size(usize),
    #[error("Could not PC slice ROM: {0}")]
    SlicePc(PcSlice),
    #[error("Could not SNES slice ROM: {0}")]
    SliceSnes(SnesSlice),
    #[error("Could not decompress ROM slice:\n- {0}")]
    Decompress(DecompressionError),
    #[error("Could not parse ROM slice")]
    Parse,
}

#[derive(Debug, Error)]
pub enum RomParseError {
    #[error("ROM error:\n- {0}")]
    BadRom(RomError),
    #[error("Invalid GFX file {0:X}:\n- {1}")]
    GfxFile(usize, GfxFileParseError),
    #[error("Parsing internal header failed:\n- {0}")]
    InternalHeader(InternalHeaderParseError),
    #[error("File IO Error")]
    IoError,
    #[error("Failed to parse level {0:#X}:\n- {1}")]
    Level(usize, LevelParseError),
    #[error("Failed to read secondary entrance {0:#X}:\n- {1}")]
    SecondaryEntrance(usize, RomError),
    #[error("Could not parse color palettes:\n- {0}")]
    ColorPalettes(ColorPaletteParseError),
    #[error("Could not parse Map16 tiles:\n- {0}")]
    Map16Tilesets(TilesetParseError),
}

#[derive(Debug, Error)]
#[error("Could not parse Map16 tiles at:\n- {0}")]
pub struct TilesetParseError(pub SnesSlice);

pub type ParseErr<'a> = nom::Err<nom::error::Error<&'a [u8]>>;

// -------------------------------------------------------------------------------------------------

impl From<LcLz2Error> for DecompressionError {
    fn from(e: LcLz2Error) -> Self {
        DecompressionError::LcLz2(e)
    }
}

impl From<LcRle1Error> for DecompressionError {
    fn from(e: LcRle1Error) -> Self {
        DecompressionError::LcRle1(e)
    }
}
