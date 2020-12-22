use crate::{
    addr::snes_to_pc,
    internal_header::MapMode,
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

pub struct Level {
    _primary_header: PrimaryHeader,
    _layer1: Layer1,
}

impl Level {
    pub fn from_rom_data(
        rom_data: &[u8],
        level_num: usize,
        map_mode: MapMode,
    ) -> IResult<&[u8], Self> {
        let snes_to_pc = snes_to_pc::decide(map_mode);

        let (ph, _layer1) = {
            let l1_ptr_addr = snes_to_pc(pointer_tables::LAYER1_DATA + (3 * level_num)).unwrap();
            let (_, ph_addr) = preceded!(rom_data, take!(l1_ptr_addr), le_u24)?;

            let (l1_input, ph_input) = preceded!(rom_data,
                take!(snes_to_pc(ph_addr as usize).unwrap()), take!(PRIMARY_HEADER_SIZE))?;

            (ph_input, l1_input)
        };

        let (_, primary_header) = PrimaryHeader::parse(ph)?;

        Ok((rom_data, Level {
            _primary_header: primary_header,
            _layer1: Layer1 {}
        }))
    }
}