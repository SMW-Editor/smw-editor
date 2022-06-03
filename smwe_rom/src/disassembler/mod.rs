// Some of the disassembler code is "borrowed" from DiztinGUIsh, an SNES ROM disassembler and debugger written in C#.
// https://github.com/Dotsarecool/DiztinGUIsh

pub mod binary_block;
pub mod instruction;
pub mod jump_tables;
pub mod opcodes;
pub mod processor;
pub mod registers;

use std::{
    collections::{BTreeMap, HashSet},
    fmt::{Debug, Formatter, Write},
    sync::Arc,
};

use itertools::Itertools;
use smallvec::{smallvec, SmallVec};

use crate::{
    disassembler::{
        binary_block::{BinaryBlock, CodeBlock},
        jump_tables::{
            get_jump_table_from_rom,
            EXECUTE_PTR_LONG_TRAMPOLINE_ADDR,
            EXECUTE_PTR_TRAMPOLINE_ADDR,
            JUMP_TABLES,
        },
        processor::Processor,
    },
    snes_utils::addr::{Addr, AddrPc, AddrSnes},
    Rom,
    RomInternalHeader,
};

// -------------------------------------------------------------------------------------------------

pub struct RomDisassembly {
    rom_bytes:  Arc<[u8]>,
    /// Start index, Block data
    pub chunks: Vec<(AddrPc, BinaryBlock)>,
}

struct RomAssemblyWalker<'r> {
    rom:        &'r Rom,
    /// Start index, Block data
    pub chunks: Vec<(AddrPc, BinaryBlock)>,

    // Algorithm state
    analysed_chunks: BTreeMap<AddrPc, (AddrPc, usize)>,

    // Temporary until code scanning
    remaining_code_starts: Vec<RomAssemblyWalkerStep>,
    analysed_code_starts:  HashSet<AddrPc>,
}

#[derive(Clone)]
struct RomAssemblyWalkerStep {
    code_start:   AddrPc,
    processor:    Processor,
    entrance:     AddrSnes,
    return_stack: SmallVec<[AddrPc; 32]>,
    next_steps:   Vec<RomAssemblyWalkerStep>,
}

type Result<T> = std::result::Result<T, ()>;

enum BlockFindResult {
    Found { range_start: AddrPc, range_end: AddrPc, range_vec_idx: usize },
    MissingWithNext { next_start: AddrPc },
    Missing,
}

// -------------------------------------------------------------------------------------------------

impl RomDisassembly {
    pub fn new(rom: &Rom, rih: &RomInternalHeader) -> Self {
        let mut walker = RomAssemblyWalker::new(rom, rih);
        walker.full_analysis().unwrap();
        Self { rom_bytes: Arc::clone(&rom.0), chunks: walker.chunks }
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
                    for i in code.instructions.iter() {
                        let ibytes = &self.rom_bytes[i.offset.0..][..i.opcode.instruction_size()];
                        write!(f, "${:6}   {:<20} # ", i.offset, i.display())?;
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

impl<'r> RomAssemblyWalker<'r> {
    fn new(rom: &'r Rom, rih: &RomInternalHeader) -> Self {
        let remaining_code_starts = [AddrSnes::MIN, EXECUTE_PTR_TRAMPOLINE_ADDR, EXECUTE_PTR_LONG_TRAMPOLINE_ADDR]
            .iter()
            .chain(rih.interrupt_vectors.iter())
            .map(|&addr| RomAssemblyWalkerStep {
                code_start:   AddrPc::try_from(addr).unwrap(),
                processor:    Processor::new(),
                entrance:     addr,
                return_stack: smallvec![],
                next_steps:   vec![],
            })
            .collect_vec();

        Self {
            rom,
            chunks: Default::default(),
            analysed_chunks: Default::default(),
            // Temporary until code scanning
            remaining_code_starts,
            analysed_code_starts: HashSet::with_capacity(256),
        }
    }

    fn full_analysis(&mut self) -> Result<()> {
        while let Some(step) = self.remaining_code_starts.pop() {
            self.analysis_step(step)?;
        }
        self.cleanup()?;
        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        self.chunks.push((AddrPc(self.rom.0.len()), BinaryBlock::EndOfRom));
        self.chunks.sort_by_key(|(address, _)| address.0);
        let mut dedup_chunks = Vec::with_capacity(self.chunks.len());
        for (_group_pc, mut chunk_group) in
            &std::mem::take(&mut self.chunks).into_iter().group_by(|(address, _)| address.0)
        {
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
        self.chunks = dedup_chunks;
        Ok(())
    }

    fn print_backtrace(
        &self, code_start: AddrPc, mut entrance: AddrSnes, processor: &Processor, code_block: &CodeBlock,
    ) {
        eprintln!("!!!! Code error backtrace start, block at {code_start}, M: {}", processor.p_reg.m_flag());
        code_block.instructions.iter().for_each(|i| eprintln!(" {}", i.display_with_flags()));
        while entrance != AddrSnes::MIN {
            let entrance_pc: AddrPc = entrance.try_into().unwrap();
            let (_, &(_, block_idx)) = self.analysed_chunks.range(entrance_pc..).next().unwrap();
            let block = self.chunks[block_idx].1.code_block().unwrap();
            eprintln!(
                "Next backtrace block at {:?}, M: {}, X: {}, entrance: {entrance:?}",
                block.instructions[0].offset, block.instructions[0].m_flag, block.instructions[0].x_flag,
            );
            block.instructions.iter().for_each(|i| eprintln!(" {}", i.display_with_flags()));
            if entrance == block.entrances[0] {
                break;
            }
            entrance = block.entrances[0];
        }
    }

    fn analysis_step(&mut self, step: RomAssemblyWalkerStep) -> Result<()> {
        let RomAssemblyWalkerStep { code_start, mut processor, entrance, return_stack, next_steps } = step;

        let mut next_known_start = self.rom.0.len();
        match self.find_block_of(code_start) {
            BlockFindResult::Found { range_start, range_end, range_vec_idx } => {
                if code_start != range_start {
                    self.split_block_at(range_start, range_end, range_vec_idx, code_start, entrance);
                }
                for mut next_step in next_steps {
                    next_step.processor =
                        self.chunks[range_vec_idx].1.code_block().unwrap().final_processor_state.clone();
                    self.enqueue_step(next_step);
                }
                return Ok(());
            }
            BlockFindResult::MissingWithNext { next_start } => {
                next_known_start = next_start.0;
            }
            BlockFindResult::Missing => {
                // no-op
            }
        }

        eprintln!(
            "analysing {code_start} to {:?}, M: {}, X: {}, entrance: {entrance:?}",
            AddrPc(next_known_start),
            processor.p_reg.m_flag(),
            processor.p_reg.x_flag(),
        );

        let (mut code_block, addr_after_block) =
            CodeBlock::from_bytes(code_start, &self.rom.0[code_start.0..next_known_start], &mut processor);
        code_block.entrances.push(entrance);

        let last_instruction = code_block.instructions.last().unwrap_or_else(|| {
            self.print_backtrace(code_start, entrance, &processor, &code_block);
            panic!("Empty (invalid) code block at {code_start}")
        });

        for mut next_step in next_steps {
            next_step.processor = code_block.final_processor_state.clone();
            self.enqueue_step(next_step);
        }

        let mut next_covered = false;
        if last_instruction.can_branch() {
            for i in code_block.instructions.iter() {
                eprintln!(" {}", i.display_with_flags());
            }

            let last_snes: AddrSnes = last_instruction.offset.try_into().unwrap();
            let mut next_instructions = last_instruction.next_instructions(last_snes).to_vec();
            let is_jump_table = next_instructions
                .iter()
                .any(|&t| t == EXECUTE_PTR_TRAMPOLINE_ADDR || t == EXECUTE_PTR_LONG_TRAMPOLINE_ADDR);
            let mut return_addr = None;
            let mut new_return_stack = return_stack.clone();
            if is_jump_table {
                next_instructions.clear();

                // The M and X flags are getting set in the `ExecutePtr` and `ExecutePtrLong` trampolines.
                processor.p_reg.0 |= 0x30;

                let jump_table_addr = AddrSnes::try_from_lorom(addr_after_block).unwrap();
                match JUMP_TABLES.iter().find(|t| t.begin == jump_table_addr) {
                    Some(&jtv) => {
                        let addresses = get_jump_table_from_rom(self.rom, jtv).unwrap();
                        for addr in addresses.into_iter().filter(|a| a.absolute() != 0) {
                            let addr_pc: AddrPc = addr.try_into().unwrap();
                            eprintln!("from jump table: {code_start:?} to {addr_pc:?}");
                            next_instructions.push(addr);
                        }
                    }
                    None => {
                        log::warn!("Could not find jump table at {jump_table_addr:?}");
                    }
                }
            } else if last_instruction.is_subroutine_call() {
                let next_instruction = last_instruction.offset + AddrPc(last_instruction.opcode.instruction_size());
                new_return_stack.push(next_instruction);
                return_addr = last_instruction.return_instruction(last_snes);
            } else if last_instruction.is_subroutine_return() {
                let return_addr = new_return_stack.pop().expect("Return address stack underflow");
                next_instructions.push(return_addr.try_into().unwrap());
            }

            for next_target in next_instructions {
                if let Ok(next_pc) = AddrPc::try_from(next_target) {
                    if next_pc.0 >= self.rom.0.len() {
                        eprintln!("Invalid next PC encountered when parsing basic code block starting at {code_start:?}, at final instruction {last_instruction:?}");
                        self.chunks.push((code_start, BinaryBlock::Code(code_block)));
                        self.analysed_chunks.insert(addr_after_block, (code_start, self.chunks.len() - 1));
                        if !next_covered {
                            self.chunks.push((addr_after_block, BinaryBlock::Unknown));
                        }
                        return Err(());
                    }

                    if next_pc == addr_after_block {
                        next_covered = true;
                    }

                    eprintln!("exit from {} to {next_target:?}", last_instruction.display_with_flags());
                    code_block.exits.push(next_target);
                    let return_step = RomAssemblyWalkerStep {
                        code_start:   return_addr.map(|a| a.try_into().unwrap()).unwrap_or_default(),
                        processor:    processor.clone(),
                        entrance:     next_pc.try_into().unwrap(),
                        return_stack: return_stack.clone(),
                        next_steps:   vec![],
                    };

                    if self.analysed_code_starts.insert(next_pc) {
                        eprintln!("new code block from {code_start:?} to {next_pc:?}");
                        self.remaining_code_starts.push(RomAssemblyWalkerStep {
                            code_start:   next_pc,
                            processor:    processor.clone(),
                            entrance:     code_start.try_into().unwrap(),
                            return_stack: new_return_stack.clone(),
                            next_steps:   if return_addr.is_some() { vec![return_step.clone()] } else { vec![] },
                        });
                    } else {
                        // TODO: Add entrance to matching code block
                        // Handle return from subroutine
                        if let Some(return_addr) = return_addr {
                            let return_pc: AddrPc = return_addr.try_into().unwrap();
                            if let BlockFindResult::Found { range_vec_idx, .. } = self.find_block_of(return_pc) {
                                // Already fully analysed
                                let block = self.chunks[range_vec_idx].1.code_block().unwrap();
                                let processor = block.final_processor_state.clone();
                                self.enqueue_step(RomAssemblyWalkerStep { processor, ..return_step.clone() });
                            } else {
                                // In analysis queue
                                let step = self.remaining_code_starts.iter_mut().find(|s| s.code_start == next_pc);
                                if let Some(step) = step {
                                    step.next_steps.push(return_step.clone());
                                } else {
                                    log::warn!("Wrong state: couldn't find a place to handle return from subroutine {next_pc:?} to {return_addr:?}");
                                }
                            }
                        }
                    }
                } else {
                    log::warn!("Wrong address of next target: {next_target:06X}");
                }
            }
        }

        self.chunks.push((code_start, BinaryBlock::Code(code_block)));
        self.analysed_chunks.insert(addr_after_block, (code_start, self.chunks.len() - 1));
        if !next_covered {
            self.chunks.push((addr_after_block, BinaryBlock::Unknown));
        }

        Ok(())
    }

    /// (range end, (range start, range vec idx))
    /// Error variant: next known start
    fn find_block_of(&self, instruction: AddrPc) -> BlockFindResult {
        if let Some((&range_end, &(range_start, range_vec_idx))) = self.analysed_chunks.range(instruction + 1..).next()
        {
            if instruction >= range_start && instruction < range_end {
                BlockFindResult::Found { range_end, range_start, range_vec_idx }
            } else {
                BlockFindResult::MissingWithNext { next_start: range_start }
            }
        } else {
            BlockFindResult::Missing
        }
    }

    fn enqueue_step(&mut self, step: RomAssemblyWalkerStep) {
        if self.analysed_code_starts.insert(step.code_start) {
            self.remaining_code_starts.push(step);
        }
    }

    /// Returns: index of the first block (second block's index remains unchanged)
    fn split_block_at(
        &mut self, range_start: AddrPc, range_end: AddrPc, range_vec_idx: usize, middle_start: AddrPc,
        entrance: AddrSnes,
    ) -> usize {
        eprintln!("split at {middle_start}");
        // jump into the middle of a block, split it in two
        let (original_pc, mut original_block) =
            std::mem::replace(&mut self.chunks[range_vec_idx], (range_start, BinaryBlock::Unknown));
        let CodeBlock {
            instructions: original_instructions,
            exits: original_exits,
            entrances: original_entrances,
            entry_processor_state,
            final_processor_state,
        } = std::mem::take(original_block.code_block_mut().expect("Found jump into the middle of a non-code section"));
        assert_eq!(original_pc, range_start);

        let mut first_block = CodeBlock {
            instructions: Vec::with_capacity(original_instructions.len() / 2),
            exits: vec![middle_start.try_into().unwrap()],
            entrances: original_entrances,
            entry_processor_state,
            final_processor_state: Default::default(),
        };
        let mut second_block = CodeBlock {
            instructions: Vec::with_capacity(original_instructions.len() / 2),
            exits: original_exits,
            entrances: vec![entrance],
            entry_processor_state: Default::default(),
            final_processor_state,
        };
        for imeta in original_instructions.into_iter() {
            if imeta.offset < middle_start { &mut first_block } else { &mut second_block }.instructions.push(imeta);
        }
        second_block.entrances.push(first_block.instructions.last().unwrap().offset.try_into().unwrap());
        first_block.recalculate_final_processor_state();
        second_block.entry_processor_state = first_block.final_processor_state.clone();

        self.chunks.push((range_start, BinaryBlock::Code(first_block)));
        self.chunks[range_vec_idx] = (middle_start, BinaryBlock::Code(second_block));
        self.analysed_chunks.remove(&range_end);
        self.analysed_chunks.insert(range_end, (middle_start, range_vec_idx));
        self.analysed_chunks.insert(middle_start, (range_start, self.chunks.len() - 1));
        self.analysed_code_starts.insert(middle_start);
        self.chunks.len() - 1
    }
}
