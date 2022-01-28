use std::{
    collections::{BTreeMap, HashSet},
    fmt::{Debug, Display, Formatter, Write},
    sync::Arc,
};

use itertools::Itertools;
use smallvec::{smallvec, SmallVec};

use crate::{
    disassembler::{
        instruction::Instruction,
        jump_tables::{get_jump_table_from_rom, JUMP_TABLES},
        processor::Processor,
    },
    snes_utils::addr::{Addr, AddrPc, AddrSnes},
    Rom,
};

// -------------------------------------------------------------------------------------------------

pub enum DataKind {
    Empty,
    Graphics,
    InternalRomHeader,
    JumpTable,
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
    pub entrances:         Vec<AddrSnes>,
}

#[derive(Clone, Copy, Debug)]
pub struct InstructionMeta {
    pub offset:      AddrPc,
    pub instruction: Instruction,
    pub m_flag:      bool,
    pub x_flag:      bool,
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

impl Display for InstructionMeta {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", if self.m_flag { 'M' } else { 'm' })?;
        self.instruction.display(self.offset, self.x_flag, self.m_flag).fmt(f)
    }
}

impl RomDisassembly {
    pub fn new(rom: &Rom) -> Self {
        let rom_bytes = Arc::clone(&rom.0);
        let mut chunks: Vec<(AddrPc, BinaryBlock)> = Vec::with_capacity(64);
        // Chunk end -> Chunk start, index in vec
        let mut analysed_chunks: BTreeMap<AddrPc, (AddrPc, usize)> = Default::default();
        // Temporary until code scanning
        // TODO: Debug maximum small vec size
        let mut remaining_code_starts: Vec<(AddrPc, Processor, AddrSnes, SmallVec<[AddrPc; 32]>)> =
            Vec::with_capacity(JUMP_TABLES.len() + 1);
        let mut analysed_code_starts = HashSet::with_capacity(256);

        remaining_code_starts.push((AddrPc::MIN, Processor::new(), AddrSnes::MIN, smallvec![]));
        // TODO: Return stack: support multiple places a subroutine is called from

        'analysis_loop: while let Some((code_start, mut processor, entrance, mut return_stack)) =
            remaining_code_starts.pop()
        {
            let mut next_known_start = rom_bytes.len();
            if let Some((&range_end, &(range_start, range_vec_idx))) = analysed_chunks.range(code_start + 1..).next() {
                if code_start >= range_start && code_start < range_end {
                    // already analysed
                    if code_start != range_start {
                        let middle_start = code_start;
                        eprintln!("split at {middle_start}");
                        // jump into the middle of a block, split it in two
                        let (original_pc, mut original_block) =
                            std::mem::replace(&mut chunks[range_vec_idx], (range_start, BinaryBlock::Unknown));
                        let CodeBlock {
                            instruction_metas: original_instructions,
                            exits: original_exits,
                            entrances: original_entrances,
                        } = std::mem::take(
                            original_block.code_block_mut().expect("Found jump into the middle of a non-code section"),
                        );
                        assert_eq!(original_pc, range_start);

                        let mut first_block = CodeBlock {
                            instruction_metas: Vec::with_capacity(original_instructions.len() / 2),
                            exits:             vec![middle_start.try_into().unwrap()],
                            entrances:         original_entrances,
                        };
                        let mut second_block = CodeBlock {
                            instruction_metas: Vec::with_capacity(original_instructions.len() / 2),
                            exits:             original_exits,
                            entrances:         vec![entrance],
                        };
                        for imeta in original_instructions.into_iter() {
                            if imeta.offset < middle_start { &mut first_block } else { &mut second_block }
                                .instruction_metas
                                .push(imeta);
                        }
                        second_block
                            .entrances
                            .push(first_block.instruction_metas.last().unwrap().offset.try_into().unwrap());

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
            {
                let nkssnes = AddrPc(next_known_start);
                eprintln!(
                    "analysing {code_start} to {nkssnes:?} M:{} X:{} entrance:{:?}",
                    processor.p_reg.m_flag(),
                    processor.p_reg.x_flag(),
                    entrance
                );
            }
            let (mut code_block, rest) =
                CodeBlock::from_bytes(code_start, &rom_bytes[code_start.0..next_known_start], &mut processor);
            code_block.entrances.push(entrance);
            let last_instruction = code_block.instruction_metas.last().unwrap_or_else(|| {
                eprintln!("!!!! Code error backtrace start, block at {code_start} M:{}", processor.p_reg.m_flag());
                code_block.instruction_metas.iter().for_each(|i| eprintln!(" {i}"));
                let mut entrance = entrance;
                while entrance != AddrSnes::MIN {
                    let entrance_pc: AddrPc = entrance.try_into().unwrap();
                    let (_, &(_, block_idx)) = analysed_chunks.range(entrance_pc..).next().unwrap();
                    let block = chunks[block_idx].1.code_block().unwrap();
                    eprintln!(
                        "Next backtrace block at {:?} M:{} [e={entrance:?}]",
                        block.instruction_metas[0].offset, block.instruction_metas[0].m_flag
                    );
                    block.instruction_metas.iter().for_each(|i| eprintln!(" {i}"));
                    if entrance == block.entrances[0] {
                        break;
                    }
                    entrance = block.entrances[0];
                }
                panic!("Empty (invalid) code block at {code_start}")
            });
            let mut next_covered = false;
            if last_instruction.instruction.opcode.mnemonic.can_branch() {
                let last_snes: AddrSnes = last_instruction.offset.try_into().unwrap();
                for meta in code_block.instruction_metas.iter() {
                    eprintln!("{meta}");
                }
                let mut next_instructions = last_instruction.instruction.next_instructions(last_snes);
                let is_jump_table = next_instructions.iter().any(|&AddrSnes(t)| t == 0x0086DF || t == 0x0086FA);
                if is_jump_table {
                    let jump_table_addr = AddrSnes::try_from_lorom(rest).unwrap();
                    match JUMP_TABLES.iter().find(|t| t.begin == jump_table_addr) {
                        Some(&jtv) => {
                            let addresses = get_jump_table_from_rom(rom, jtv).unwrap();
                            for addr in addresses.into_iter().filter(|a| a.0 & 0xFFFF != 0) {
                                let addr = AddrPc::try_from_lorom(addr).unwrap();
                                if analysed_code_starts.insert(addr) {
                                    eprintln!("from jump table: {code_start:?} to {addr:?}");
                                    remaining_code_starts.push((
                                        addr,
                                        processor.clone(),
                                        code_start.try_into().unwrap(),
                                        return_stack.clone(),
                                    ));
                                } else {
                                    // TODO: Add entrance to matching code block
                                }
                            }
                        }
                        None => {
                            log::warn!("Could not find jump table at {jump_table_addr:?}");
                        }
                    }
                } else {
                    if last_instruction.instruction.opcode.mnemonic.is_subroutine_call() {
                        let next_instruction =
                            last_instruction.offset + AddrPc(last_instruction.instruction.opcode.instruction_size());
                        return_stack.push(next_instruction);
                    } else if last_instruction.instruction.opcode.mnemonic.is_subroutine_return() {
                        let return_addr = return_stack.pop().expect("Address stack underflow");
                        next_instructions.push(return_addr.try_into().unwrap());
                    }
                    for next_target in next_instructions {
                        if let Ok(next_pc) = AddrPc::try_from(next_target) {
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
                            eprintln!("exit from {last_instruction} to {next_target:?}");
                            code_block.exits.push(next_target);
                            if analysed_code_starts.insert(next_pc) {
                                eprintln!("from {code_start:?} to {next_pc:?}");
                                remaining_code_starts.push((
                                    next_pc,
                                    processor.clone(),
                                    code_start.try_into().unwrap(),
                                    return_stack.clone(),
                                ));
                            } else {
                                // TODO: Add entrance to matching code block
                            }
                        } else {
                            log::warn!("Wrong address of next target: {next_target:06X}");
                        }
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
                        write!(f, "${:6}   {:<20} # ", i.offset, i.instruction.display(i.offset, i.x_flag, i.m_flag))?;
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
            };
            instruction_metas.push(meta);
            rest = new_rest;
            addr = addr + i.opcode.instruction_size();
            processor.execute(i);
            if i.opcode.mnemonic.can_branch() {
                break;
            }
        }
        (Self { instruction_metas, exits: Vec::with_capacity(2), entrances: Vec::with_capacity(2) }, addr)
    }
}
