mod data;

pub use data::JUMP_TABLES;
use nom::{
    combinator::map,
    multi::many1,
    number::complete::{le_u16, le_u24},
};

use crate::{
    error::RomError,
    snes_utils::{addr::AddrSnes, rom_slice::SnesSlice},
    Rom,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct JumpTableView {
    pub begin:     AddrSnes,
    /// Number of pointers (16-bit or 24-bit ints), not bytes.
    pub length:    usize,
    pub long_ptrs: bool,
}

impl JumpTableView {
    pub const fn new(begin: AddrSnes, length: usize, long_ptrs: bool) -> Self {
        Self { begin, length, long_ptrs }
    }
}

pub fn get_jump_table_from_rom(rom: &Rom, jump_table_view: JumpTableView) -> Result<Vec<AddrSnes>, RomError> {
    let ptr_size: usize = if jump_table_view.long_ptrs { 3 } else { 2 };
    let slice = SnesSlice::new(jump_table_view.begin, jump_table_view.length * ptr_size);

    let jump_table = if jump_table_view.long_ptrs {
        let parser = many1(map(le_u24, |a| AddrSnes(a as usize)));
        rom.view().slice_lorom(slice)?.parse(parser)?
    } else {
        // 16-bit address implies the same bank number as the jump table's address.
        let parser = many1(map(le_u16, |a| AddrSnes(a as usize) | (jump_table_view.begin & 0xFF0000)));
        rom.view().slice_lorom(slice)?.parse(parser)?
    };

    Ok(jump_table)
}
