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
pub struct CodeBlock {
    pub instructions: Vec<(AddrPc, Instruction)>,
}

#[derive(Clone)]
pub struct DataBlock {}

#[derive(Clone)]
pub enum BinaryBlock {
    Code(CodeBlock),
    Data(DataBlock),
    Unused,
    Unknown,
    EndOfRom,
}

pub struct RomDisassembly {
    rom_bytes:  Arc<[u8]>,
    /// Start index, Block data
    pub chunks: Vec<(AddrPc, BinaryBlock)>,
}

// -------------------------------------------------------------------------------------------------

impl RomDisassembly {
    pub fn new(rom_bytes: Arc<[u8]>) -> Self {
        let mut chunks = Vec::with_capacity(64);
        // Temporary until code scanning
        let (first_code, rest) = CodeBlock::from_bytes(AddrPc(0), &rom_bytes);
        chunks.push((AddrPc(0), BinaryBlock::Code(first_code)));
        if rest.0 != rom_bytes.len() {
            chunks.push((rest, BinaryBlock::Unknown));
        }
        chunks.push((AddrPc(rom_bytes.len()), BinaryBlock::EndOfRom));
        Self { rom_bytes, chunks }
    }

    pub fn rom_bytes(&self) -> &[u8] {
        &self.rom_bytes
    }
}

impl CodeBlock {
    /// Returns parsed code block and the address of the next byte after the block end
    pub fn from_bytes(base: AddrPc, bytes: &[u8]) -> (Self, AddrPc) {
        let mut instructions = Vec::with_capacity(bytes.len() / 2);
        let mut rest = bytes;
        let mut addr = base;
        while let Ok((i, new_rest)) = Instruction::parse(rest) {
            instructions.push((addr, i));
            rest = new_rest;
            addr = AddrPc(addr.0 + i.opcode.instruction_size());
        }
        (Self { instructions }, addr)
    }
}
