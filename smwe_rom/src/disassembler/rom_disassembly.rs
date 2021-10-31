use std::{collections::BTreeMap, sync::Arc};

use crate::{
    disassembler::instruction::Instruction,
    snes_utils::addr::{AddrPc, AddrSnes},
};

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

#[derive(Clone)]
pub struct CodeBlockMetadata {}

#[derive(Clone)]
pub struct DataBlockMetadata {}

#[derive(Clone)]
pub enum BinaryBlock {
    Code(CodeBlockMetadata),
    Data(DataBlockMetadata),
    Unused,
    Unknown,
    EndOfRom,
}

pub struct RomDisassembly {
    rom_bytes:  Arc<[u8]>,
    /// Map Start index -> Block data
    pub chunks: BTreeMap<AddrPc, BinaryBlock>,
}

// -------------------------------------------------------------------------------------------------

impl RomDisassembly {
    pub fn new(rom_bytes: Arc<[u8]>) -> Self {
        let mut chunks = BTreeMap::new();
        chunks.insert(AddrPc(0), BinaryBlock::Unknown);
        chunks.insert(AddrPc(rom_bytes.len()), BinaryBlock::EndOfRom);
        Self { rom_bytes, chunks }
    }

    pub fn rom_bytes(&self) -> &[u8] {
        &self.rom_bytes
    }
}
