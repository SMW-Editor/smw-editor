use std::sync::Arc;

use crate::{
    disassembler::{instruction::Instruction, processor::Processor},
    snes_utils::addr::AddrPc,
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
    pub instruction_metas: Vec<InstructionMeta>,
}

#[derive(Clone, Copy)]
pub struct InstructionMeta {
    pub offset:      AddrPc,
    pub instruction: Instruction,
    pub m_flag:      bool,
    pub x_flag:      bool,
    pub direct_page: u16,
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
        let mut processor = Processor::new();
        let mut chunks = Vec::with_capacity(64);
        // Temporary until code scanning
        let (first_code, rest) = CodeBlock::from_bytes(AddrPc(0), &rom_bytes, &mut processor);
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
    pub fn from_bytes(base: AddrPc, bytes: &[u8], processor: &mut Processor) -> (Self, AddrPc) {
        let mut instruction_metas = Vec::with_capacity(bytes.len() / 2);
        let mut rest = bytes;
        let mut addr = base;
        while let Ok((i, new_rest)) = Instruction::parse(rest, processor.p_reg) {
            instruction_metas.push(InstructionMeta {
                offset:      addr,
                instruction: i,
                m_flag:      processor.p_reg.m_flag(),
                x_flag:      processor.p_reg.x_flag(),
                direct_page: processor.dp_reg.0,
            });
            rest = new_rest;
            addr = addr + i.opcode.instruction_size();
            processor.execute(i);
        }
        (Self { instruction_metas }, addr)
    }
}
