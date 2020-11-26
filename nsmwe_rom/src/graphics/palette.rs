use crate::{
    addr::AddressSNES,
    graphics::color::{
        read_bgr16_as_rgba,
        Rgba,
    },
};

pub const PALETTE_PTR_TABLE: AddressSNES = 0x0EF600;
pub const COLORS_IN_PALETTE: usize = 16 * 16;
pub const PALETTE_DATA_SIZE: usize = 0x202;

pub struct Palette {
    back_area_color: Rgba,
    actual_palette: [Rgba; COLORS_IN_PALETTE],
}

impl Palette {
    pub fn from_data(data: &[u8]) -> Result<Self, String> {
        assert_eq!(PALETTE_DATA_SIZE, data.len());

        let back_area_color = read_bgr16_as_rgba(data, 0)?;

        let mut actual_palette = [[0.; 4]; COLORS_IN_PALETTE];
        for (i, color) in actual_palette.iter_mut().enumerate() {
            *color = read_bgr16_as_rgba(data, 2 * (i + 1))?;
        }

        Ok(Palette { back_area_color, actual_palette })
    }
}