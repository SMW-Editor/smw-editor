use std::{
    collections::{BTreeMap, HashSet},
    fmt::{Debug, Formatter, Write},
    sync::Arc,
};

use itertools::Itertools;

use crate::{
    disassembler::{instruction::Instruction, processor::Processor},
    snes_utils::addr::{Addr, AddrPc, AddrSnes},
    Rom,
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
    // will be deleted once we identify all data kinds in SMW ROM
    NotYetDetermined,
    OverworldLayer1,
    OverworldLayer2,
    OverworldSpriteLayer,
    SoundSample,
    Text,
}

#[derive(Default, Clone)]
pub struct CodeBlock {
    pub instruction_metas: Vec<InstructionMeta>,
    pub exits:             Vec<AddrSnes>,
}

#[derive(Clone, Copy, Debug)]
pub struct InstructionMeta {
    pub offset:      AddrPc,
    pub instruction: Instruction,
    pub m_flag:      bool,
    pub x_flag:      bool,
    pub direct_page: u16,
}

#[derive(Default, Clone)]
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
    pub fn new(rom: &Rom) -> Self {
        let rom_bytes = Arc::clone(&rom.0);
        let mut chunks: Vec<(AddrPc, BinaryBlock)> = Vec::with_capacity(64);
        // Chunk end -> Chunk start, index in vec
        let mut analysed_chunks: BTreeMap<AddrPc, (AddrPc, usize)> = Default::default();
        // Temporary until code scanning
        let mut remaining_code_starts = Vec::with_capacity(16);
        let mut analysed_code_starts = HashSet::with_capacity(256);
        remaining_code_starts.push((AddrPc::MIN, Processor::new()));
        analysed_code_starts.insert(AddrPc::MIN);
        'analysis_loop: while let Some((code_start, mut processor)) = remaining_code_starts.pop() {
            let mut next_known_start = rom_bytes.len();
            dbg!(code_start);
            dbg!(&analysed_chunks);
            if let Some((&range_end, &(range_start, range_vec_idx))) =
                analysed_chunks.range(code_start + AddrPc(1)..).next()
            {
                if code_start >= range_start && code_start < range_end {
                    // already analysed
                    if code_start != range_start {
                        let middle_start = code_start;
                        // jump into the middle of a block, split it in two
                        let (original_pc, mut original_block) =
                            std::mem::replace(&mut chunks[range_vec_idx], (range_start, BinaryBlock::Unknown));
                        let CodeBlock { instruction_metas: original_instructions, exits: original_exits } =
                            std::mem::take(
                                original_block
                                    .code_block_mut()
                                    .expect("Found jump into the middle of a non-code section"),
                            );
                        assert_eq!(original_pc, range_start);

                        let mut first_block = CodeBlock {
                            instruction_metas: Vec::with_capacity(original_instructions.len() / 2),
                            exits:             vec![middle_start.try_into().unwrap()],
                        };
                        let mut second_block = CodeBlock {
                            instruction_metas: Vec::with_capacity(original_instructions.len() / 2),
                            exits:             original_exits,
                        };
                        for imeta in original_instructions.into_iter() {
                            if imeta.offset < middle_start { &mut first_block } else { &mut second_block }
                                .instruction_metas
                                .push(imeta);
                        }

                        chunks.push((range_start, BinaryBlock::Code(first_block)));
                        chunks[range_vec_idx] = (middle_start, BinaryBlock::Code(second_block));
                        analysed_chunks.remove(&range_end);
                        analysed_chunks.insert(range_end, (middle_start, range_vec_idx));
                        analysed_chunks.insert(middle_start, (range_start, chunks.len() - 1));
                        analysed_code_starts.insert(middle_start);
                    }
                    continue;
                } else {
                    next_known_start = range_start.0;
                }
            }
            let (mut code_block, rest) =
                CodeBlock::from_bytes(code_start, &rom_bytes[code_start.0..next_known_start], &mut processor);
            let last_instruction = code_block.instruction_metas.last().expect("Empty (invalid) code block");
            let mut next_covered = false;
            if last_instruction.instruction.opcode.mnemonic.can_branch() {
                let last_snes: AddrSnes = last_instruction.offset.try_into().unwrap();
                for next_target in last_instruction.instruction.next_instructions(last_snes, processor.dp_reg.0) {
                    let next_pc = AddrPc::try_from(next_target).unwrap();
                    if next_pc.0 >= rom_bytes.len() {
                        eprintln!("Invalid next PC encountered when parsing basic code block starting at {:?}, at final instruction {:?}", code_start, last_instruction);
                        chunks.push((code_start, BinaryBlock::Code(code_block)));
                        analysed_chunks.insert(rest, (code_start, chunks.len() - 1));
                        if !next_covered {
                            chunks.push((rest, BinaryBlock::Unknown));
                        }
                        break 'analysis_loop;
                    }
                    if next_pc == rest {
                        next_covered = true;
                    }
                    code_block.exits.push(next_target);
                    if analysed_code_starts.insert(next_pc) {
                        remaining_code_starts.push((next_pc, processor.clone()));
                    }
                }
            }
            chunks.push((code_start, BinaryBlock::Code(code_block)));
            analysed_chunks.insert(rest, (code_start, chunks.len() - 1));
            if !next_covered {
                chunks.push((rest, BinaryBlock::Unknown));
            }
        }
        chunks.push((AddrPc(rom_bytes.len()), BinaryBlock::EndOfRom));
        chunks.sort_by_key(|(address, _)| address.0);
        let mut dedup_chunks = Vec::with_capacity(chunks.len());
        for (_group_pc, mut chunk_group) in &chunks.into_iter().group_by(|(address, _)| address.0) {
            let first = chunk_group.next().unwrap();
            dedup_chunks.push(first);
            let final_chunk = dedup_chunks.last_mut().unwrap();
            for chunk in chunk_group {
                if matches!(final_chunk.1, BinaryBlock::Unknown) {
                    *final_chunk = chunk;
                } else if matches!(chunk.1, BinaryBlock::Unknown) {
                    continue;
                } else {
                    panic!("Multiple chunks generated at address {}", final_chunk.0);
                }
            }
        }
        Self { rom_bytes, chunks: dedup_chunks }
    }

    pub fn rom_bytes(&self) -> &[u8] {
        &self.rom_bytes
    }
}

impl Debug for RomDisassembly {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for ((address, block), (next_address, _)) in self.chunks.iter().tuple_windows::<(_, _)>() {
            writeln!(f, " #### CHUNK {} .. {}", address, next_address)?;
            match block {
                BinaryBlock::Code(code) => {
                    for exit in code.exits.iter() {
                        writeln!(f, "# Exit: {}", exit)?;
                    }
                    for i in code.instruction_metas.iter() {
                        let ibytes = &self.rom_bytes[i.offset.0..][..i.instruction.opcode.instruction_size()];
                        write!(
                            f,
                            "${:6}   {:<20} # ",
                            i.offset,
                            i.instruction.display(i.offset, i.x_flag, i.m_flag, i.direct_page)
                        )?;
                        for &byte in ibytes {
                            write!(f, "{:02x} ", byte)?;
                        }
                        f.write_char('\n')?;
                    }
                }
                BinaryBlock::Data(_data) => writeln!(f, "# Data")?,
                BinaryBlock::Unused => writeln!(f, "# Unused")?,
                BinaryBlock::Unknown => writeln!(f, "# Unknown")?,
                BinaryBlock::EndOfRom => writeln!(f, "# End of ROM")?,
            }
        }
        Ok(())
    }
}

impl BinaryBlock {
    pub fn type_name(&self) -> &'static str {
        use BinaryBlock::*;
        match self {
            Code(_) => "Code",
            Data(_) => "Data",
            Unused => "Unused",
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
        let mut instruction_metas = Vec::with_capacity(bytes.len() / 2);
        let mut addr = base;
        let mut rest = bytes;
        while let Ok((i, new_rest)) = Instruction::parse(rest, processor.p_reg) {
            let meta = InstructionMeta {
                offset:      addr,
                instruction: i,
                m_flag:      processor.p_reg.m_flag(),
                x_flag:      processor.p_reg.x_flag(),
                direct_page: processor.dp_reg.0,
            };
            instruction_metas.push(meta);
            rest = new_rest;
            addr = addr + i.opcode.instruction_size();
            processor.execute(i);
            if i.opcode.mnemonic.can_branch() {
                break;
            }
        }
        (Self { instruction_metas, exits: Vec::new() }, addr)
    }
}
