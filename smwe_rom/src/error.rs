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
    #[error("Isolating Internal ROM Header:\n- {0}")]
    IsolatingData(RomError),

    #[error("Reading checksum and complement at LoROM location")]
    ReadLoRomChecksum,
    #[error("Reading checksum and complement at HiROM location")]
    ReadHiRomChecksum,
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
    #[error("Isolating data:\n- {0}")]
    IsolatingData(RomError),
    #[error("Decompressing data:\n- {0}")]
    DecompressingData(DecompressionError),
    #[error("Parsing tile")]
    ParsingTile,
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

    #[error("Reading address of Layer1:\n- {0}")]
    Layer1AddressRead(RomError),
    #[error("Reading address of Layer2:\n- {0}")]
    Layer2AddressRead(RomError),
    #[error("Reading address of Sprite data:\n- {0}")]
    SpriteAddressRead(RomError),

    #[error("Isolating Layer1 data")]
    Layer1Isolate,
    #[error("Isolating Layer2 data:\n- {0}")]
    Layer2Isolate(RomError),
    #[error("Isolating Sprite data:\n- {0}")]
    SpriteIsolate(RomError),

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

    #[error("Parsing Layer1 object data")]
    Layer1Parse,
}

#[derive(Debug, Error)]
pub enum RomError {
    #[error("Invalid ROM size: {0} ({0:#x})")]
    Size(usize),

    #[error("Invalid PC slice: {0}")]
    SlicePc(PcSlice),
    #[error("Invalid LoROM slice: {0}")]
    SliceLoRom(SnesSlice),
    #[error("Invalid HiROM slice: {0}")]
    SliceHiRom(SnesSlice),

    #[error("Could not parse PC slice: {0}")]
    ParsePc(PcSlice),
    #[error("Could not parse LoROM slice: {0}")]
    ParseLoRom(SnesSlice),
    #[error("Could not parse HiROM slice: {0}")]
    ParseHiRom(SnesSlice),

    #[error("Address conversion in LoROM slicing:\n- {0}")]
    AddressSliceLoRom(AddressError),
    #[error("Address conversion in HiROM slicing:\n- {0}")]
    AddressSliceHiRom(AddressError),
    #[error("Address conversion in LoROM parsing:\n- {0}")]
    AddressParseLoRom(AddressError),
    #[error("Address conversion in HiROM parsing:\n- {0}")]
    AddressParseHiRom(AddressError),
}

#[derive(Debug, Error)]
pub enum RomParseError {
    #[error("ROM error:\n- {0}")]
    BadRom(RomError),
    #[error("ROM doesn't contain PC address {0:#x}")]
    BadAddress(usize),
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
    #[error("Could not parse color palettes")]
    ColorPalettes(ColorPaletteParseError),
}

pub type ParseErr<'a> = nom::Err<nom::error::Error<&'a [u8]>>;
