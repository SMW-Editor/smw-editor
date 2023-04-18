// Some of the disassembler code is "borrowed" from DiztinGUIsh, an SNES ROM disassembler and debugger written in C#.
// https://github.com/Dotsarecool/DiztinGUIsh

pub mod binary_block;
pub mod instruction;
pub mod jump_tables;
pub mod opcodes;
pub mod processor;
pub mod registers;
pub mod serialization;

use std::{
    cell::RefCell,
    cmp::Ordering,
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    fmt::{Debug, Formatter, Write},
    ops::Deref,
    rc::Rc,
};

use itertools::Itertools;
use thiserror::Error;

use crate::{
    disassembler::{
        binary_block::{BinaryBlock, CodeBlock, DataBlock, DataKind},
        instruction::Instruction,
        jump_tables::{
            get_jump_table_from_rom,
            EXECUTE_PTR_LONG_TRAMPOLINE_ADDR,
            EXECUTE_PTR_TRAMPOLINE_ADDR,
            JUMP_TABLES,
            NON_CODE_JUMP_ADDRESSES,
        },
        processor::Processor,
        serialization::LineKind,
    },
    snes_utils::{
        addr::{Addr, AddrPc, AddrSnes},
        rom::{RomViewWithErrorMapper, SnesSliced},
        rom_slice::SnesSlice,
    },
    Rom,
    RomError,
    RomInternalHeader,
};

// -------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, Error)]
pub enum DisassemblyError {
    #[error("Cannot find a block that returns from subroutine starting at ${0:?}")]
    SubroutineWithoutReturn(AddrSnes),
    #[error("Invalid next PC encountered when parsing basic code block starting at {0:?}, at final instruction {1:?}")]
    InvalidAddrInCodeBlock(AddrPc, Instruction),
}

// -------------------------------------------------------------------------------------------------

pub struct RomDisassembly {
    pub rom:        Rom,
    /// Start index, Block data
    pub chunks:     Vec<(AddrPc, BinaryBlock)>,
    pub code_lines: Vec<LineKind>,

    cached_data_blocks: HashSet<DataBlock>,
}

struct RomAssemblyWalker {
    rom:        Rom,
    /// Start index, Block data
    pub chunks: Vec<(AddrPc, BinaryBlock)>,

    // Algorithm state
    analysed_chunks: BTreeMap<AddrPc, (AddrPc, usize)>,

    // Temporary until code scanning
    remaining_steps:      VecDeque<RomAssemblyWalkerStep>,
    analysed_code_starts: HashSet<AddrPc>,
    /// Subroutine start -> addresses of call return points
    subroutine_returns:   HashMap<AddrPc, Vec<AddrPc>>,
    analysed_subroutines: HashMap<AddrPc, Rc<RefCell<SubroutineAnalysisState>>>,
}

#[derive(Clone)]
enum RomAssemblyWalkerStep {
    BasicBlock(StepBasicBlock),
    Subroutine(StepSubroutine),
}

#[derive(Clone)]
struct StepSubroutine {
    code_start: AddrPc,
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
struct SubroutineAnalysisState {
    /// (block address, block index)
    code_blocks:           Vec<usize>,
    analysed_blocks:       HashSet<AddrPc>,
    remaining_blocks:      Vec<AddrPc>,
    final_processor_state: Processor,
}

type Result<T> = std::result::Result<T, DisassemblyError>;

enum BlockFindResult {
    Found { range_start: AddrPc, range_end: AddrPc, range_vec_idx: usize },
    MissingWithNext { next_start: AddrPc },
    Missing,
}

// -------------------------------------------------------------------------------------------------

impl RomDisassembly {
    pub fn new(rom: Rom, rih: &RomInternalHeader) -> Self {
        let mut walker = RomAssemblyWalker::new(rom.clone(), rih);
        walker.full_analysis().unwrap();
        Self {
            rom,
            chunks: walker.chunks,
            cached_data_blocks: HashSet::new(),
            code_lines: {
                std::fs::read_to_string("ROM/out.json")
                    .ok()
                    .and_then(|s| match serde_json::from_str(&s) {
                        Ok(s) => Some(s),
                        Err(e) => {
                            log::error!("{e}");
                            None
                        }
                    })
                    .unwrap_or(Vec::new())
            },
        }
    }

    pub fn rom_bytes(&self) -> &[u8] {
        &self.rom.0
    }

    /// Parses a data block and marks it with given kind and size determined by the `parse` function. `parse` returns
    /// the parsed data and number of ROM bytes consumed by the parser.
    pub fn parse_and_mark_data<EM, ET, RT, PF>(
        &mut self, block_start: AddrSnes, data_kind: DataKind, error_mapper: EM, parse: PF,
    ) -> std::result::Result<RT, ET>
    where
        PF: FnOnce(RomViewWithErrorMapper<'_, EM, ET, SnesSliced<'_>>) -> std::result::Result<(RT, usize), ET>,
        EM: Fn(RomError) -> ET + Clone,
    {
        let mut block = DataBlock { slice: SnesSlice::new(block_start, usize::MAX), kind: data_kind };
        let rom_view = self.rom_slice_at_block(block, error_mapper.clone())?;
        let (ret, size) = parse(rom_view)?;

        // Replace data block if its size hasn't been determined yet.
        if self.cached_data_blocks.contains(&block) {
            self.cached_data_blocks.remove(&block);
            block.slice.size = size;
            self.split_unknown_block_with(block, &error_mapper)?;
            self.cached_data_blocks.insert(block);
        }

        Ok(ret)
    }

    /// Requests a data block from disassembly. If the data block hasn't been determined before, this function finds
    /// a `BinaryBlock::Unknown` containing the area of `data_block` and splits or replaces it with `BinaryBlock::Data`,
    /// depending on the size and location of `data_block`. The newly established data block is also marked with its
    /// `kind`.
    ///
    /// If requested `data_block` overlaps with two already established blocks or is inside a `BinaryBlock::Code`, this
    /// function returns an error from `error_mapper`.
    ///
    /// If requested `data_block` has unknown size (its `slice` is infinite) it means that the intention is to later
    /// update its size with another request for a block with the same location and kind.
    ///
    /// If a block has been determined at the same location as the requested `data_block` but they have different sizes,
    /// the larger one gets chosen.
    ///
    /// Panics when the block at requested location was previously determined and has a different kind.
    pub fn rom_slice_at_block<EM, ET>(
        &mut self, mut data_block: DataBlock, error_mapper: EM,
    ) -> std::result::Result<RomViewWithErrorMapper<'_, EM, ET, SnesSliced<'_>>, ET>
    where
        EM: Fn(RomError) -> ET,
    {
        if !self.cached_data_blocks.contains(&data_block) {
            match self.cached_data_blocks.iter().find(|b| b.slice.begin == data_block.slice.begin) {
                None => {
                    // Requested block hasn't been established yet.
                    self.split_unknown_block_with(data_block, &error_mapper)?;
                }
                Some(&old_data_block) => {
                    assert_eq!(old_data_block.kind, data_block.kind);
                    // `data_block` and `old_block` differ in size, which means that either:
                    // 1. `old_block` is infinite - the block at this address was previously
                    // requested when its size was not known, but now is and `data_block` is used to
                    // update that information.
                    // 2. `data_block` is infinite - the block at this address has known size but
                    // the caller doesn't know it yet. That means we can use `old_block` to slice
                    // the ROM instead.
                    if !data_block.slice.is_infinite() {
                        if old_data_block.slice.is_infinite() {
                            self.cached_data_blocks.remove(&old_data_block);
                            self.split_unknown_block_with(data_block, &error_mapper)?;
                        } else if data_block.slice.size > old_data_block.slice.size {
                            self.cached_data_blocks.remove(&old_data_block);
                            let block_addr_pc = AddrPc::try_from_lorom(old_data_block.slice.begin).unwrap();
                            self.chunks.retain(|(addr, _)| *addr != block_addr_pc);
                            self.split_unknown_block_with(data_block, &error_mapper)?;
                        } else {
                            data_block = old_data_block;
                        }
                    } else {
                        // `data_block` is infinite and `old_block` is not: that means a block of
                        // unspecified size is being requested, but since the size has been
                        // previously established, we can return it.
                        data_block = old_data_block;
                    }
                }
            }

            self.cached_data_blocks.insert(data_block);
        }

        self.rom.with_error_mapper(error_mapper).slice_lorom(data_block.slice)
    }

    fn split_unknown_block_with<EM, ET>(
        &mut self, data_block: DataBlock, error_mapper: &EM,
    ) -> std::result::Result<(), ET>
    where
        EM: Fn(RomError) -> ET,
    {
        enum SplitType {
            None,
            Start(usize),
            Middle(usize),
        }

        let addr = AddrPc::try_from(data_block.slice.begin).unwrap();

        let mut split_type = SplitType::None;
        let mut found = false;
        for (i, ((begin, block), (next_begin, next_block))) in self.chunks.iter().tuple_windows::<(_, _)>().enumerate()
        {
            let next_chunk_start = AddrSnes::try_from_lorom(*next_begin).unwrap();
            match begin.cmp(&addr) {
                Ordering::Equal => {
                    match block {
                        BinaryBlock::Code(_) | BinaryBlock::EndOfRom => {
                            log::error!("Inside code block at {begin:?}");
                            return Err(error_mapper(RomError::DataBlockNotFound(data_block)));
                        }
                        BinaryBlock::Data(found_block) => {
                            if *found_block != data_block {
                                log::error!("Data block mismatch: {found_block:?}");
                                return Err(error_mapper(RomError::DataBlockNotFound(data_block)));
                            }
                        }
                        BinaryBlock::Unknown => {
                            if data_block.slice.contains(next_chunk_start) {
                                match next_block {
                                    BinaryBlock::Code(_) => {
                                        log::error!("Requested data block overlaps with the next code block at: {next_chunk_start:?}");
                                    }
                                    BinaryBlock::Data(next_data_block) => {
                                        log::error!("Requested data block overlaps with the next data block:");
                                        eprintln!("> data_block = {data_block:?}");
                                        eprintln!("> next_block = {next_data_block:?}");
                                    }
                                    BinaryBlock::Unknown => {
                                        log::error!("Requested data block overlaps with the next unknown block at: {next_chunk_start:?}");
                                    }
                                    BinaryBlock::EndOfRom => {
                                        log::error!("Requested data block doesn't fit in the ROM");
                                    }
                                }
                                return Err(error_mapper(RomError::DataBlockNotFound(data_block)));
                            }
                            if !data_block.slice.is_infinite() {
                                split_type = SplitType::Start(i);
                            }
                        }
                    }
                    found = true;
                    break;
                }
                Ordering::Less => {
                    let begin_pc = AddrPc::try_from_lorom(data_block.slice.begin).unwrap();
                    if (*begin..*next_begin).contains(&begin_pc) {
                        if !data_block.slice.is_infinite() {
                            split_type = SplitType::Middle(i);
                        }
                        found = true;
                        break;
                    }
                }
                Ordering::Greater => {
                    // skip
                }
            }
        }

        if !found {
            log::error!("Data block not found");
            return Err(error_mapper(RomError::DataBlockNotFound(data_block)));
        }

        let begin_pc = AddrPc::try_from_lorom(data_block.slice.begin).unwrap();
        match split_type {
            SplitType::Start(index) => {
                self.chunks[index].0 += data_block.slice.size as u32;
                self.chunks.insert(index, (begin_pc, BinaryBlock::Data(data_block)));
            }
            SplitType::Middle(index) => {
                let data_end = begin_pc + data_block.slice.size as u32;
                let next_begin = self.chunks[index + 1].0;
                assert!(data_end <= next_begin, "data_end = {data_end}, next_begin = {next_begin}, index = {index}");
                self.chunks.insert(index + 1, (begin_pc, BinaryBlock::Data(data_block)));
                if data_end < next_begin {
                    self.chunks.insert(index + 2, (data_end, BinaryBlock::Unknown));
                }
            }
            SplitType::None => {
                // No-op
            }
        }

        Ok(())
    }
}

impl Debug for RomDisassembly {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for ((address, block), (next_address, _)) in self.chunks.iter().tuple_windows::<(_, _)>() {
            writeln!(f, " #### CHUNK {address} .. {next_address}")?;
            match block {
                BinaryBlock::Code(code) => {
                    for exit in code.exits.iter() {
                        writeln!(f, "# Exit: {exit}")?;
                    }
                    for i in code.instructions.iter() {
                        let ibytes = &self.rom.0[i.offset.as_index()..][..i.opcode.instruction_size()];
                        write!(f, "${:6}   {:<20} # ", i.offset, i.display())?;
                        for &byte in ibytes {
                            write!(f, "{byte:02x} ")?;
                        }
                        f.write_char('\n')?;
                    }
                }
                BinaryBlock::Data(_data) => writeln!(f, "# Data")?,
                BinaryBlock::Unknown => writeln!(f, "# Unknown")?,
                BinaryBlock::EndOfRom => writeln!(f, "# End of ROM")?,
            }
        }
        Ok(())
    }
}

impl RomAssemblyWalker {
    fn new(rom: Rom, rih: &RomInternalHeader) -> Self {
        let remaining_steps = [AddrSnes::MIN, EXECUTE_PTR_TRAMPOLINE_ADDR, EXECUTE_PTR_LONG_TRAMPOLINE_ADDR]
            .iter()
            .chain(rih.interrupt_vectors.iter())
            .filter(|a| a.0 != 0xFFFF)
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
        self.cleanup();
        Ok(())
    }

    fn cleanup(&mut self) {
        self.chunks.push((AddrPc(self.rom.0.len() as _), BinaryBlock::EndOfRom));
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
        let sub = self
            .analysed_subroutines
            .entry(step.code_start)
            .or_insert_with(|| {
                Rc::new(RefCell::new(SubroutineAnalysisState {
                    code_blocks:           Vec::with_capacity(32),
                    analysed_blocks:       HashSet::with_capacity(32),
                    remaining_blocks:      vec![step.code_start],
                    final_processor_state: Processor::new(),
                }))
            })
            .clone();
        let mut sub = sub.deref().borrow_mut();

        while let Some(curr_code_start) = sub.remaining_blocks.pop() {
            match self.find_analysed_chunk_at(curr_code_start) {
                BlockFindResult::Found { range_vec_idx, .. } => {
                    sub.code_blocks.push(range_vec_idx);

                    let block = self.chunks[range_vec_idx].1.code_block().unwrap();
                    let last_instruction = *block.instructions.last().unwrap();

                    if last_instruction.uses_jump_table() {
                        continue;
                    }

                    let addr_after_block = last_instruction.offset + last_instruction.opcode.instruction_size() as u32;
                    let exits = block.exits.iter();

                    // The zero exits case happens when a JSR or JSL's destination is not in ROM but e.g. in RAM.
                    if exits.len() != 0 {
                        if last_instruction.is_subroutine_call() {
                            let sub_location = AddrPc::try_from(exits.as_slice()[0]).unwrap();
                            self.subroutine_returns.entry(sub_location).or_default().push(addr_after_block);
                            if step.sub_exists_in_call_hierarchy(sub_location) {
                                self.enqueue_basic_block(StepBasicBlock {
                                    code_start: addr_after_block,
                                    processor:  sub.final_processor_state.clone(),
                                    entrance:   step.entrance,
                                });
                            } else {
                                let next_step = StepSubroutine {
                                    code_start: sub_location,
                                    entrance:   last_instruction.offset.try_into().unwrap(),
                                    caller:     Some(Box::new(step.clone())),
                                };
                                if self.enqueue_subroutine(next_step) {
                                    sub.remaining_blocks.push(addr_after_block);
                                    return Ok(());
                                }
                            }
                        } else if !last_instruction.is_subroutine_return() {
                            let pending_blocks = exits
                                .clone()
                                .map(|&a| AddrPc::try_from(a).unwrap())
                                .filter(|&a| sub.analysed_blocks.insert(a))
                                .collect_vec();
                            sub.remaining_blocks.extend(pending_blocks.into_iter());
                        }
                    }

                    if !last_instruction.is_single_path_leap() && sub.analysed_blocks.insert(addr_after_block) {
                        sub.remaining_blocks.push(addr_after_block);
                    }
                }
                _ => {
                    sub.remaining_blocks.push(curr_code_start);
                    self.remaining_steps.push_back(RomAssemblyWalkerStep::Subroutine(step));
                    return Ok(());
                }
            }
        }

        match sub.code_blocks.iter().find(|&&idx| {
            let last_ins = self.chunks[idx].1.code_block().unwrap().instructions.last().unwrap();
            last_ins.is_subroutine_return() || last_ins.uses_jump_table()
        }) {
            Some(&returning_block_index) => {
                sub.final_processor_state =
                    self.chunks[returning_block_index].1.code_block().unwrap().final_processor_state.clone();
            }
            None => return Err(DisassemblyError::SubroutineWithoutReturn(step.code_start.try_into().unwrap())),
        };

        if let Some(caller) = step.caller {
            self.enqueue_subroutine(*caller);
        }

        // Subroutines in jump tables don't have returns, or rather we don't need to analyse them.
        if let Some(returns) = self.subroutine_returns.get(&step.code_start) {
            for return_addr in returns.clone().into_iter() {
                self.enqueue_basic_block(StepBasicBlock {
                    code_start: return_addr,
                    processor:  sub.final_processor_state.clone(),
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
                next_known_start = next_start.as_index();
            }
            BlockFindResult::Missing => {
                // no-op
            }
        }

        let (mut code_block, addr_after_block) =
            CodeBlock::from_bytes(code_start, &self.rom.0[code_start.as_index()..next_known_start], &mut processor);
        code_block.entrances.push(entrance);

        let last_instruction = code_block.instructions.last().unwrap_or_else(|| {
            self.print_backtrace(code_start, entrance, &processor, &code_block);
            panic!("Empty (invalid) code block at {code_start}")
        });

        let mut next_covered = false;
        if last_instruction.can_change_program_counter() {
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
                        let addresses = get_jump_table_from_rom(&self.rom, jtv).unwrap();
                        for addr in addresses.into_iter().filter(|a| a.absolute() != 0) {
                            if !NON_CODE_JUMP_ADDRESSES.contains(&addr) {
                                next_instructions.push(addr);
                            }
                        }
                        self.chunks.push((
                            addr_after_block,
                            BinaryBlock::Data(DataBlock {
                                slice: SnesSlice::new(jtv.begin, jtv.length),
                                kind:  if jtv.long_ptrs { DataKind::JumpTableLong } else { DataKind::JumpTableShort },
                            }),
                        ));
                    }
                }
            } else if last_instruction.is_subroutine_call() {
                let mut step_following_block = StepBasicBlock {
                    code_start: addr_after_block,
                    processor:  processor.clone(),
                    entrance:   code_start.try_into().unwrap(),
                };

                match AddrPc::try_from(next_instructions[0]) {
                    Ok(sub_start) => {
                        self.subroutine_returns.entry(sub_start).or_default().push(addr_after_block);
                        if let Some(sub) = self.analysed_subroutines.get(&sub_start) {
                            if sub.deref().borrow().is_complete() {
                                step_following_block.processor = sub.deref().borrow().final_processor_state.clone();
                                self.enqueue_basic_block(step_following_block);
                            }
                        }
                    }
                    Err(_) => {
                        // The subroutine being called might be located in RAM and in such case we can assume the
                        // state of the processor to be unchanged.
                        self.enqueue_basic_block(step_following_block);
                    }
                }
            }

            for &next_target_snes in next_instructions.iter() {
                match AddrPc::try_from(next_target_snes) {
                    Err(_) => log::warn!("Wrong address of next target: {next_target_snes:06X}"),
                    Ok(next_target_pc) if next_target_pc.as_index() >= self.rom.0.len() => {
                        let last_instruction = *last_instruction;
                        self.chunks.push((code_start, BinaryBlock::Code(code_block)));
                        self.analysed_chunks.insert(addr_after_block, (code_start, self.chunks.len() - 1));
                        if !next_covered {
                            self.chunks.push((addr_after_block, BinaryBlock::Unknown));
                        }
                        return Err(DisassemblyError::InvalidAddrInCodeBlock(code_start, last_instruction));
                    }
                    Ok(next_target_pc) => {
                        if next_target_pc == addr_after_block {
                            next_covered = true;
                        }

                        code_block.exits.push(next_target_snes);

                        self.enqueue_basic_block(StepBasicBlock {
                            code_start: next_target_pc,
                            processor:  processor.clone(),
                            entrance:   code_start.try_into().unwrap(),
                        });
                    }
                }
            }

            if is_jump_table || last_instruction.is_subroutine_call() {
                for sub_start in next_instructions.into_iter() {
                    if let Ok(code_start) = AddrPc::try_from(sub_start) {
                        self.enqueue_subroutine(StepSubroutine {
                            code_start,
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

impl SubroutineAnalysisState {
    pub fn is_complete(&self) -> bool {
        self.remaining_blocks.is_empty()
    }
}
