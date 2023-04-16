use nom::{combinator::map, multi::many0, number::complete::le_u16};
use thiserror::Error;

use crate::{
    disassembler::{
        binary_block::{DataBlock, DataKind},
        RomDisassembly,
    },
    snes_utils::{addr::AddrSnes, rom_slice::SnesSlice},
};

// -------------------------------------------------------------------------------------------------

#[derive(Debug, Error)]
#[error("Could not parse AnimatedTileData table.")]
pub struct AnimatedTileDataParseError;

// -------------------------------------------------------------------------------------------------

const ANIMATED_TILE_DATA_TABLE: SnesSlice = SnesSlice::new(AddrSnes(0x05B999), 416);

// -------------------------------------------------------------------------------------------------

pub struct AnimatedTileData {
    pub addresses: Vec<AddrSnes>,
}

impl AnimatedTileData {
    pub fn parse(disasm: &mut RomDisassembly) -> Result<Self, AnimatedTileDataParseError> {
        let block = DataBlock { slice: ANIMATED_TILE_DATA_TABLE, kind: DataKind::AnimatedTileData };
        let addresses = disasm
            .rom_slice_at_block(block, |_| AnimatedTileDataParseError)?
            .parse(many0(map(le_u16, |a| AddrSnes(a as _).with_bank(0x7E))))?;
        Ok(Self { addresses })
    }
}
