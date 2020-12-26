use crate::{
    addr::{AddrPc, AddrSnes},
    level::{
        layer1::Layer1,
        pointer_tables,
        primary_header::{
            PrimaryHeader,
            PRIMARY_HEADER_SIZE,
        },
    },
};

use nom::{
    IResult,
    number::complete::le_u24,
    preceded,
    take,
};

use std::convert::TryFrom;

pub struct Level {
    _primary_header: PrimaryHeader,
    _layer1: Layer1,
}

impl Level {
    pub fn from_rom_data(
        rom_data: &[u8],
        level_num: usize,
    ) -> IResult<&[u8], Self> {
        let (_layer1, ph) = {
            let l1_ptr_addr = AddrPc::try_from(pointer_tables::LAYER1_DATA + (3 * level_num))
                .unwrap();
            let (_, ph_addr) = preceded!(rom_data, take!(l1_ptr_addr.0), le_u24)?;
            let ph_addr: usize = AddrPc::try_from(AddrSnes(ph_addr as usize))
                .unwrap()
                .into();
            preceded!(rom_data, take!(ph_addr), take!(PRIMARY_HEADER_SIZE))?
        };

        let (_, primary_header) = PrimaryHeader::parse(ph)?;

        Ok((rom_data, Level {
            _primary_header: primary_header,
            _layer1: Layer1 {}
        }))
    }
}