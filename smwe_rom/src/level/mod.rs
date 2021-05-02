pub mod layer1;
pub mod pointer_tables;
pub mod primary_header;
pub mod secondary_header;

use std::convert::TryFrom;

use nom::{number::complete::le_u24, preceded, take, IResult};

pub use self::{
    layer1::Layer1,
    primary_header::{PrimaryHeader, PRIMARY_HEADER_SIZE},
    secondary_header::SecondaryHeader,
};
use crate::addr::{AddrPc, AddrSnes};

pub const LEVEL_COUNT: usize = 0x200;

#[derive(Clone)]
pub struct Level {
    pub primary_header: PrimaryHeader,
    pub _layer1:        Layer1,
}

impl Level {
    pub fn from_rom_data(rom_data: &[u8], level_num: usize) -> IResult<&[u8], Self> {
        let (_layer1, ph) = {
            let l1_ptr_addr = AddrPc::try_from(pointer_tables::LAYER1_DATA + (3 * level_num)).unwrap();
            let (_, ph_addr) = preceded!(rom_data, take!(l1_ptr_addr.0), le_u24)?;
            let ph_addr: usize = AddrPc::try_from(AddrSnes(ph_addr as usize)).unwrap().into();
            preceded!(rom_data, take!(ph_addr), take!(PRIMARY_HEADER_SIZE))?
        };

        let (_, primary_header) = PrimaryHeader::parse(ph)?;

        Ok((rom_data, Level { primary_header, _layer1: Layer1 {} }))
    }
}
