use crate::disassembler::instruction::Instruction;

// -------------------------------------------------------------------------------------------------

pub enum DataKind {
    Empty,
    Graphics,
    InternalRomHeader,
    LevelBackgroundLayer,
    LevelObjectLayer,
    LevelSpriteLayer,
    Music,
    NotYetDetermined, // will be deleted once we identify all data kinds in SMW ROM
    OverworldLayer1,
    OverworldLayer2,
    OverworldSpriteLayer,
    SoundSample,
    Text,
}

pub enum BinaryChunk {
    Code(Instruction),
    Data(DataKind, Vec<u8>),
}

pub struct RomDisassembly {
    chunks: Vec<BinaryChunk>,
}

// -------------------------------------------------------------------------------------------------


