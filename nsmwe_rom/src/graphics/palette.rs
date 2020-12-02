use crate::{
    addr::AddressSnes,
    graphics::color::{
        read_bgr16_as_rgba,
        Rgba,
    },
};

use nom::IResult;

pub const PALETTE_PTR_TABLE: AddressSnes = 0x0EF600;
pub const COLORS_IN_PALETTE: usize = 16 * 16;
pub const PALETTE_DATA_SIZE: usize = 0x202;

pub struct Palette {
    back_area_color: Rgba,
    actual_palette: [Rgba; COLORS_IN_PALETTE],
}

impl Palette {
    pub fn parse(data: &[u8]) -> IResult<&[u8], Palette> {
        assert_eq!(PALETTE_DATA_SIZE, data.len());

        let (_, back_area_color) = read_bgr16_as_rgba(&data[0..=1])?;

        let mut actual_palette = [[0.; 4]; COLORS_IN_PALETTE];
        for color in actual_palette.iter_mut() {
            let (_, read_color) = read_bgr16_as_rgba(&data)?;
            *color = read_color;
        }

        Ok((data, Palette { back_area_color, actual_palette }))
    }
}