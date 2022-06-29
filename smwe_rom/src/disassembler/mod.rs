// Some of the disassembler code is "borrowed" from DiztinGUIsh, an SNES ROM disassembler and debugger written in C#.
// https://github.com/Dotsarecool/DiztinGUIsh

pub mod binary_block;
pub mod instruction;
pub mod jump_tables;
pub mod opcodes;
pub mod processor;
pub mod registers;

use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    fmt::{Debug, Formatter, Write},
    sync::Arc,
};

use itertools::Itertools;

use crate::{
    disassembler::{
        binary_block::{BinaryBlock, CodeBlock},
        jump_tables::{
            get_jump_table_from_rom,
            EXECUTE_PTR_LONG_TRAMPOLINE_ADDR,
            EXECUTE_PTR_TRAMPOLINE_ADDR,
            JUMP_TABLES,
            NON_CODE_JUMP_ADDRESSES,
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
    remaining_steps:      VecDeque<RomAssemblyWalkerStep>,
    analysed_code_starts: HashSet<AddrPc>,
    /// Subroutine start -> addresses of call return points
    subroutine_returns:   HashMap<AddrPc, Vec<AddrPc>>,
    analysed_subroutines: HashMap<AddrPc, Subroutine>,
}

#[derive(Clone)]
enum RomAssemblyWalkerStep {
    BasicBlock(StepBasicBlock),
    Subroutine(StepSubroutine),
}

#[derive(Clone)]
struct StepSubroutine {
    code_start: AddrPc,
    processor:  Processor,
    entrance:   AddrSnes,
    caller:     Option<Box<StepSubroutine>>,
}

#[derive(Clone)]
struct StepBasicBlock {
    code_start: AddrPc,
    processor:  Processor,
    entrance:   AddrSnes,
}

#[derive(Clone)]
struct Subroutine {
    /// (block address, block index)
    code_blocks:           Vec<(AddrPc, usize)>,
    final_processor_state: Processor,
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
        let remaining_steps = [AddrSnes::MIN, EXECUTE_PTR_TRAMPOLINE_ADDR, EXECUTE_PTR_LONG_TRAMPOLINE_ADDR]
            .iter()
            .chain(rih.interrupt_vectors.iter())
            .map(|&addr| StepBasicBlock {
                code_start: AddrPc::try_from(addr).unwrap(),
                processor:  Processor::new(),
                entrance:   addr,
            })
            .map(RomAssemblyWalkerStep::BasicBlock)
            .collect();

        Self {
            rom,
            chunks: Default::default(),
            analysed_chunks: Default::default(),
            remaining_steps,
            analysed_code_starts: HashSet::with_capacity(256),
            subroutine_returns: HashMap::with_capacity(256),
            analysed_subroutines: HashMap::with_capacity(256),
        }
    }

    fn full_analysis(&mut self) -> Result<()> {
        while let Some(step) = self.remaining_steps.pop_front() {
            match step {
                RomAssemblyWalkerStep::BasicBlock(step) => self.analyse_basic_block(step)?,
                RomAssemblyWalkerStep::Subroutine(step) => self.analyse_subroutine(step)?,
            }
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

    fn analyse_subroutine(&mut self, step: StepSubroutine) -> Result<()> {
        let mut block_indices = Vec::with_capacity(32);
        let mut analysed_blocks = HashSet::with_capacity(32);
        let mut remaining_blocks = vec![step.code_start];

        while let Some(curr_code_start) = remaining_blocks.pop() {
            let _code_start_snes = AddrSnes::try_from(curr_code_start).unwrap();
            match self.find_analysed_chunk_at(curr_code_start) {
                BlockFindResult::Found { range_vec_idx, range_start, .. } => {
                    block_indices.push((range_start, range_vec_idx));

                    let block = self.chunks[range_vec_idx].1.code_block().unwrap();
                    let last_instruction = *block.instructions.last().unwrap();

                    if last_instruction.uses_jump_table() {
                        continue;
                    }

                    let addr_after_block = last_instruction.offset + last_instruction.opcode.instruction_size();
                    let exits = block.exits.iter();

                    // The zero exits case happens when a JSR or JSL's destination is not in ROM but e.g. in RAM.
                    if exits.len() != 0 {
                        if last_instruction.is_subroutine_call() {
                            let sub_location = AddrPc::try_from(exits.as_slice()[0]).unwrap();
                            self.subroutine_returns.entry(sub_location).or_default().push(addr_after_block);
                            if !step.sub_exists_in_call_hierarchy(sub_location) {
                                let sub = StepSubroutine {
                                    code_start: sub_location,
                                    processor:  block.final_processor_state.clone(),
                                    entrance:   last_instruction.offset.try_into().unwrap(),
                                    caller:     Some(Box::new(step.clone())),
                                };
                                if self.enqueue_subroutine(sub) {
                                    return Ok(());
                                }
                            }
                        } else if !last_instruction.is_subroutine_return() {
                            let pending_blocks = exits
                                .clone()
                                .map(|&a| AddrPc::try_from(a).unwrap())
                                .filter(|&a| analysed_blocks.insert(a));
                            remaining_blocks.extend(pending_blocks);
                        }
                    }

                    if !last_instruction.is_single_path_leap() && analysed_blocks.insert(addr_after_block) {
                        remaining_blocks.push(addr_after_block);
                    }
                }
                _ => {
                    self.remaining_steps.push_back(RomAssemblyWalkerStep::Subroutine(step));
                    return Ok(());
                }
            }
        }

        let &(_, returning_block_index) = block_indices
            .iter()
            .find(|&&(_, idx)| {
                let last_ins = self.chunks[idx].1.code_block().unwrap().instructions.last().unwrap();
                last_ins.is_subroutine_return() || last_ins.uses_jump_table()
            })
            .expect("Cannot find a block that returns from subroutine.");
        let processor = self.chunks[returning_block_index].1.code_block().unwrap().final_processor_state.clone();
        self.analysed_subroutines.insert(step.code_start, Subroutine {
            final_processor_state: processor.clone(),
            code_blocks:           block_indices,
        });

        if let Some(caller) = step.caller {
            self.enqueue_subroutine(*caller);
        }

        // Subroutines in jump tables don't have returns, or rather we don't need to analyse them.
        if let Some(returns) = self.subroutine_returns.get(&step.code_start) {
            for return_addr in returns.clone().into_iter() {
                self.enqueue_basic_block(StepBasicBlock {
                    code_start: return_addr,
                    processor:  processor.clone(),
                    entrance:   step.entrance,
                });
            }
        }

        Ok(())
    }

    fn analyse_basic_block(&mut self, step: StepBasicBlock) -> Result<()> {
        let StepBasicBlock { code_start, mut processor, entrance } = step;

        let mut next_known_start = self.rom.0.len();
        match self.find_analysed_chunk_at(code_start) {
            BlockFindResult::Found { range_start, range_end, range_vec_idx } => {
                if code_start != range_start {
                    self.split_block_at(range_start, range_end, range_vec_idx, code_start, entrance);
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

        let mut next_covered = false;
        if last_instruction.can_change_program_counter() {
            for i in code_block.instructions.iter() {
                eprintln!(" {}", i.display_with_flags());
            }

            let mut next_instructions = last_instruction.next_instructions().to_vec();
            let is_jump_table = last_instruction.uses_jump_table();
            if is_jump_table {
                next_instructions.clear();

                // The M and X flags are getting set in the `ExecutePtr` and `ExecutePtrLong` trampolines.
                processor.p_reg.0 |= 0x30;

                let jump_table_addr = AddrSnes::try_from_lorom(addr_after_block).unwrap();
                match JUMP_TABLES.iter().find(|t| t.begin == jump_table_addr) {
                    None => log::warn!("Could not find jump table at {jump_table_addr:?}"),
                    Some(&jtv) => {
                        let addresses = get_jump_table_from_rom(self.rom, jtv).unwrap();
                        for addr in addresses.into_iter().filter(|a| a.absolute() != 0) {
                            if !NON_CODE_JUMP_ADDRESSES.contains(&addr) {
                                let addr_pc: AddrPc = addr.try_into().unwrap();
                                eprintln!("from jump table: {code_start:?} to {addr_pc:?}");
                                next_instructions.push(addr);
                            }
                        }
                    }
                }
            } else if last_instruction.is_subroutine_call() {
                let mut step_following_block = StepBasicBlock {
                    code_start: addr_after_block,
                    processor:  processor.clone(),
                    entrance:   code_start.try_into().unwrap(),
                };

                if let Ok(sub_start) = AddrPc::try_from(next_instructions[0]) {
                    self.subroutine_returns.entry(sub_start).or_default().push(addr_after_block);
                    if let Some(sub) = self.analysed_subroutines.get(&sub_start) {
                        step_following_block.processor = sub.final_processor_state.clone();
                        self.enqueue_basic_block(step_following_block);
                    }
                } else {
                    // The subroutine being called might be located in RAM and in such case we can assume the
                    // state of the processor to be unchanged.
                    self.enqueue_basic_block(step_following_block);
                }
            }

            for &next_target_snes in next_instructions.iter() {
                match AddrPc::try_from(next_target_snes) {
                    Err(_) => log::warn!("Wrong address of next target: {next_target_snes:06X}"),
                    Ok(next_target_pc) if next_target_pc.0 >= self.rom.0.len() => {
                        eprintln!("Invalid next PC encountered when parsing basic code block starting at {code_start:?}, at final instruction {last_instruction:?}");
                        self.chunks.push((code_start, BinaryBlock::Code(code_block)));
                        self.analysed_chunks.insert(addr_after_block, (code_start, self.chunks.len() - 1));
                        if !next_covered {
                            self.chunks.push((addr_after_block, BinaryBlock::Unknown));
                        }
                        return Err(());
                    }
                    Ok(next_target_pc) => {
                        if next_target_pc == addr_after_block {
                            next_covered = true;
                        }

                        eprintln!("exit from {} to {next_target_snes:?}", last_instruction.display_with_flags());
                        code_block.exits.push(next_target_snes);

                        if self.enqueue_basic_block(StepBasicBlock {
                            code_start: next_target_pc,
                            processor:  processor.clone(),
                            entrance:   code_start.try_into().unwrap(),
                        }) {
                            eprintln!("new code block from {code_start:?} to {next_target_pc:?}");
                        }
                    }
                }
            }

            if is_jump_table || last_instruction.is_subroutine_call() {
                for sub_start in next_instructions.into_iter() {
                    if let Ok(code_start) = AddrPc::try_from(sub_start) {
                        self.enqueue_subroutine(StepSubroutine {
                            code_start,
                            processor: processor.clone(),
                            entrance: last_instruction.offset.try_into().unwrap(),
                            caller: None,
                        });
                    }
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

    fn find_analysed_chunk_at(&self, instruction: AddrPc) -> BlockFindResult {
        match self.analysed_chunks.range(instruction + 1..).next() {
            Some((&range_end, &(range_start, range_vec_idx))) => {
                if instruction >= range_start && instruction < range_end {
                    BlockFindResult::Found { range_end, range_start, range_vec_idx }
                } else {
                    BlockFindResult::MissingWithNext { next_start: range_start }
                }
            }
            None => BlockFindResult::Missing,
        }
    }

    fn enqueue_basic_block(&mut self, step: StepBasicBlock) -> bool {
        if self.analysed_code_starts.insert(step.code_start) {
            self.remaining_steps.push_front(RomAssemblyWalkerStep::BasicBlock(step));
            true
        } else {
            false
        }
    }

    fn enqueue_subroutine(&mut self, step: StepSubroutine) -> bool {
        if !self.analysed_subroutines.contains_key(&step.code_start) {
            self.remaining_steps.push_front(RomAssemblyWalkerStep::Subroutine(step));
            true
        } else {
            false
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

impl StepSubroutine {
    pub fn sub_exists_in_call_hierarchy(&self, sub_addr: AddrPc) -> bool {
        if self.code_start == sub_addr {
            true
        } else if let Some(caller) = self.caller.clone() {
            caller.sub_exists_in_call_hierarchy(sub_addr)
        } else {
            false
        }
    }
}
