use std::{
    collections::{BTreeMap, HashSet},
    fmt::{Debug, Formatter, Write},
    sync::Arc,
};

use itertools::Itertools;

use crate::{
    disassembler::{instruction::Instruction, processor::Processor},
    snes_utils::addr::{Addr, AddrPc, AddrSnes},
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

#[derive(Clone)]
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
        // Chunk end -> Chunk start
        let mut analysed_chunks: BTreeMap<AddrPc, AddrPc> = Default::default();
        // Temporary until code scanning
        let mut remaining_code_starts = Vec::with_capacity(16);
        let mut analysed_code_starts = HashSet::with_capacity(256);
        remaining_code_starts.push((AddrPc::MIN, Processor::new()));
        analysed_code_starts.insert(AddrPc::MIN);
        'analysis_loop: while let Some((code_start, mut processor)) = remaining_code_starts.pop() {
            if let Some((&range_end, &range_start)) = analysed_chunks.range(code_start..).next() {
                if code_start >= range_start && code_start < range_end {
                    // already analysed
                    continue;
                }
            }
            let (mut code_block, rest) = CodeBlock::from_bytes(code_start, &rom_bytes[code_start.0..], &mut processor);
            let last_instruction = code_block.instruction_metas.last().expect("Empty (invalid) code block");
            let mut next_covered = false;
            if last_instruction.instruction.opcode.mnemonic.can_branch() {
                let last_snes: AddrSnes = last_instruction.offset.try_into().unwrap();
                for next_target in last_instruction.instruction.next_instructions(last_snes, processor.dp_reg.0) {
                    let next_pc = AddrPc::try_from(next_target).unwrap();
                    if next_pc.0 >= rom_bytes.len() {
                        eprintln!("Invalid next PC encountered when parsing basic code block starting at {:?}, at final instruction {:?}", code_start, last_instruction);
                        chunks.push((code_start, BinaryBlock::Code(code_block)));
                        analysed_chunks.insert(rest, code_start);
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
            analysed_chunks.insert(rest, code_start);
            if !next_covered {
                chunks.push((rest, BinaryBlock::Unknown));
            }
        }
        chunks.push((AddrPc(rom_bytes.len()), BinaryBlock::EndOfRom));
        chunks.sort_by_key(|e| e.0 .0);
        let mut dedup_chunks = Vec::with_capacity(chunks.len());
        for (_group_pc, mut chunk_group) in &chunks.into_iter().group_by(|e| e.0 .0) {
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
        for (chunk, next_chunk) in self.chunks.iter().tuple_windows::<(_, _)>() {
            writeln!(f, " #### CHUNK {} .. {}", chunk.0, next_chunk.0)?;
            match &chunk.1 {
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
