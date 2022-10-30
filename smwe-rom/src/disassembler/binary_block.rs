use crate::{
    disassembler::{instruction::Instruction, processor::Processor},
    snes_utils::{
        addr::{Addr, AddrPc, AddrSnes},
        rom_slice::SnesSlice,
    },
};

// -------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub enum BinaryBlock {
    Code(CodeBlock),
    Data(DataBlock),
    Unknown,
    EndOfRom,
}

#[derive(Default, Clone)]
pub struct CodeBlock {
    pub instructions:          Vec<Instruction>,
    pub exits:                 Vec<AddrSnes>,
    pub entrances:             Vec<AddrSnes>,
    pub entry_processor_state: Processor,
    pub final_processor_state: Processor,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct DataBlock {
    pub slice: SnesSlice,
    pub kind:  DataKind,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum DataKind {
    Empty,

    // Color palettes
    ColorPaletteCommon,
    ColorPaletteLevel,
    ColorPaletteOverworld,
    ColorPaletteOverworldLayer2Indirect1,
    ColorPaletteOverworldLayer2Indirect2,

    // Jump tables
    JumpTableShort,
    JumpTableLong,

    // Level
    LevelPointersLayer1,
    LevelPointersLayer2,
    LevelPointersSprite,
    LevelHeaderPrimary,
    LevelHeaderSecondaryByteTable,
    LevelHeaderSprites,
    LevelLayer1Objects,
    LevelLayer2Background,
    LevelLayer2Objects,
    LevelSpriteLayer,
    SecondaryEntranceTable,

    // Overworld
    OverworldLayer1,
    OverworldLayer2,
    OverworldSpriteLayer,

    // GFX lists
    GfxListObjects,
    GfxListSprites,

    // Misc.
    GfxFile,
    InternalRomHeader,
    Music,
    SoundSample,
    Text,
    Tileset,
}

// -------------------------------------------------------------------------------------------------

impl BinaryBlock {
    pub fn type_name(&self) -> &'static str {
        use BinaryBlock::*;
        match self {
            Code(_) => "Code",
            Data(_) => "Data",
            Unknown => "Unknown",
            EndOfRom => "End of ROM",
        }
    }

    pub fn code_block(&self) -> Option<&CodeBlock> {
        match self {
            Self::Code(code) => Some(code),
            _ => None,
        }
    }

    pub fn code_block_mut(&mut self) -> Option<&mut CodeBlock> {
        match self {
            Self::Code(code) => Some(code),
            _ => None,
        }
    }

    pub fn data_block(&self) -> Option<&DataBlock> {
        match self {
            Self::Data(data) => Some(data),
            _ => None,
        }
    }

    pub fn data_block_mut(&mut self) -> Option<&mut DataBlock> {
        match self {
            Self::Data(data) => Some(data),
            _ => None,
        }
    }
}

impl CodeBlock {
    /// Returns parsed code block and the address of the next byte after the block end
    pub fn from_bytes(base: AddrPc, bytes: &[u8], processor: &mut Processor) -> (Self, AddrPc) {
        let mut instructions = Vec::with_capacity(bytes.len() / 2);
        let mut addr = base;
        let mut rest = bytes;
        let original_processor = processor.clone();
        while let Ok((i, new_rest)) = Instruction::parse(rest, addr, processor.p_reg) {
            instructions.push(i);
            rest = new_rest;
            addr += i.opcode.instruction_size();
            processor.execute(i);
            if i.can_change_program_counter() {
                break;
            }
        }
        (
            Self {
                instructions,
                exits: Vec::with_capacity(2),
                entrances: Vec::with_capacity(2),
                entry_processor_state: original_processor,
                final_processor_state: processor.clone(),
            },
            addr,
        )
    }

    pub fn recalculate_final_processor_state(&mut self) {
        let mut processor = self.entry_processor_state.clone();
        for &insn in self.instructions.iter() {
            processor.execute(insn);
        }
        self.final_processor_state = processor;
    }
}

impl DataBlock {
    pub fn empty_with_kind(kind: DataKind) -> Self {
        Self { slice: SnesSlice::new(AddrSnes::MIN, 0), kind }
    }

    #[must_use]
    pub fn with_slice(mut self, slice: SnesSlice) -> Self {
        self.slice = slice;
        self
    }

    #[must_use]
    pub fn with_addr(mut self, begin: AddrSnes) -> Self {
        self.slice.begin = begin;
        self
    }

    #[must_use]
    pub fn with_size(mut self, size: usize) -> Self {
        self.slice.size = size;
        self
    }

    #[must_use]
    pub fn with_kind(mut self, kind: DataKind) -> Self {
        self.kind = kind;
        self
    }
}
