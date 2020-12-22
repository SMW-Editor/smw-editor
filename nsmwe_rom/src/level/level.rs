use crate::level::{
    layer1::Layer1,
    pointer_tables,
    primary_header::{
        PrimaryHeader,
        PRIMARY_HEADER_SIZE,
    },
};

use nom::{
    IResult,
    number::complete::le_u24,
};

pub struct Level {
    _primary_header: PrimaryHeader,
    _layer1: Layer1,
}

impl Level {
    pub fn from_rom_data(rom_data: &[u8], level_num: usize) -> IResult<&[u8], Self> {
        let (primary_header, _layer1) = {
            let layer1_ptr_addr = pointer_tables::LAYER1_DATA + (3 * level_num);
            let input = &rom_data[layer1_ptr_addr..layer1_ptr_addr + 3];

            let (_l1_input, ph_addr) = le_u24(input)?;
            let ph_addr = ph_addr as usize;

            let ph_input = &rom_data[ph_addr..ph_addr + PRIMARY_HEADER_SIZE];

            (ph_input, ())
        };

        let (_, primary_header) = PrimaryHeader::parse(primary_header)?;

        Ok((rom_data, Level {
            _primary_header: primary_header,
            _layer1: Layer1 {}
        }))
    }
}