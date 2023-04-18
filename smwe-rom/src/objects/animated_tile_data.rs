use nom::{combinator::map, multi::many0, number::complete::le_u16};
use thiserror::Error;

use crate::{
    disassembler::{
        binary_block::{DataBlock, DataKind},
        RomDisassembly,
    },
    snes_utils::{
        addr::{AddrSnes, AddrVram},
        rom_slice::SnesSlice,
    },
};

// -------------------------------------------------------------------------------------------------

#[derive(Debug, Error)]
#[error("Could not parse AnimatedTileData table.")]
pub struct AnimatedTileDataParseError;

// -------------------------------------------------------------------------------------------------

const ANIM_SRC_ADDRESSES_TABLE: SnesSlice = SnesSlice::new(AddrSnes(0x05B999), 416);
const ANIM_DST_ADDRESSES_TABLE: SnesSlice = SnesSlice::new(AddrSnes(0x05B93B), 48);

// -------------------------------------------------------------------------------------------------

pub struct AnimatedTileData {
    pub src_addresses: Vec<AddrSnes>,
    pub dst_addresses: Vec<AddrVram>,
}

impl AnimatedTileData {
    pub fn parse(disasm: &mut RomDisassembly) -> Result<Self, AnimatedTileDataParseError> {
        let src_addresses = {
            let data_block = DataBlock { slice: ANIM_SRC_ADDRESSES_TABLE, kind: DataKind::AnimatedTileData };
            disasm
                .rom_slice_at_block(data_block, |_| AnimatedTileDataParseError)?
                .parse(many0(map(le_u16, |a| AddrSnes(a as _).with_bank(0x7E))))?
        };
        let dst_addresses = {
            let data_block = DataBlock { slice: ANIM_DST_ADDRESSES_TABLE, kind: DataKind::AnimatedTileData };
            disasm
                .rom_slice_at_block(data_block, |_| AnimatedTileDataParseError)?
                .parse(many0(map(le_u16, AddrVram)))?
        };
        Ok(Self { src_addresses, dst_addresses })
    }
}
