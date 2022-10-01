#![allow(clippy::unusual_byte_groupings)]
#![allow(dead_code)]

use epaint::{Color32, Rgba};
use epaint::color::gamma_u8_from_linear_f32;

pub const ABGR1555_SIZE: usize = std::mem::size_of::<Abgr1555>();

const SNES_BGR_CHANNEL_MAX: u16 = 0b11111;

// -------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug)]
pub struct Abgr1555(pub u16);

// -------------------------------------------------------------------------------------------------

#[rustfmt::skip]
impl Abgr1555 {
    pub const TRANSPARENT: Abgr1555 = Abgr1555(0b1_00000_00000_00000);
    pub const BLACK:       Abgr1555 = Abgr1555(0b0_00000_00000_00000);
    pub const WHITE:       Abgr1555 = Abgr1555(0b0_11111_11111_11111);
    pub const RED:         Abgr1555 = Abgr1555(0b0_00000_00000_11111);
    pub const GREEN:       Abgr1555 = Abgr1555(0b0_00000_11111_00000);
    pub const BLUE:        Abgr1555 = Abgr1555(0b0_11111_00000_00000);
    pub const MAGENTA:     Abgr1555 = Abgr1555(0b0_11111_00000_11111);
}

impl Default for Abgr1555 {
    fn default() -> Self {
        Abgr1555::TRANSPARENT
    }
}

impl From<Rgba> for Abgr1555 {
    fn from(color: Rgba) -> Self {
        let cmf = SNES_BGR_CHANNEL_MAX as f32;
        let r = (color.r() * cmf).round() as u16 & SNES_BGR_CHANNEL_MAX;
        let g = (color.g() * cmf).round() as u16 & SNES_BGR_CHANNEL_MAX;
        let b = (color.b() * cmf).round() as u16 & SNES_BGR_CHANNEL_MAX;
        let a = (color.a() as u16) & 1;
        Abgr1555(((1 - a) << 0xF) | (b << 0xA) | (g << 0x5) | (r << 0x0))
    }
}

impl From<Color32> for Abgr1555 {
    fn from(color: Color32) -> Self {
        Abgr1555::from(Rgba::from(color))
    }
}

impl From<Abgr1555> for Rgba {
    fn from(color: Abgr1555) -> Self {
        let cmf = SNES_BGR_CHANNEL_MAX as f32;
        Rgba::from_srgba_unmultiplied(
            ((((color.0 >> 0x0) & SNES_BGR_CHANNEL_MAX) as f32 / cmf) * 255.0) as u8,
            ((((color.0 >> 0x5) & SNES_BGR_CHANNEL_MAX) as f32 / cmf) * 255.0) as u8,
            ((((color.0 >> 0xA) & SNES_BGR_CHANNEL_MAX) as f32 / cmf) * 255.0) as u8,
            (1 - ((color.0 >> 0xF) & 1) as u8) * 255,
        )
    }
}

impl From<Abgr1555> for Color32 {
    fn from(color: Abgr1555) -> Self {
        Color32::from(Rgba::from(color))
    }
}
