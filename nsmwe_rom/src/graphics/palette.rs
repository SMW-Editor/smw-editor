use crate::{
    addr::AddressSnes,
    graphics::color::Bgr16,
};

use nom::{
    count,
    number::complete::le_u16,
    IResult,
};

use std::convert::TryInto;

pub const PALETTE_PTR_TABLE: AddressSnes = 0x0EF600;
pub const COLORS_IN_PALETTE: usize = 16 * 16;
pub const PALETTE_DATA_SIZE: usize = 0x202;

pub struct Palette {
    back_area_color: Bgr16,
    actual_palette: [Bgr16; COLORS_IN_PALETTE],
}

impl Palette {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Palette> {
        assert_eq!(PALETTE_DATA_SIZE, input.len());

        let (input, back_area_color) = le_u16(&input[0..2])?;
        let (input, colors) = count!(input, le_u16, COLORS_IN_PALETTE)?;

        Ok((input, Palette {
            back_area_color,
            actual_palette: colors
                .try_into()
                .expect("Number of elements in Vec is incorrect."),
        }))
    }
}