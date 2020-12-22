use std::mem::size_of;

pub const BGR16_SIZE: usize = size_of::<Bgr16>();

const SNES_BGR_CHANNEL_MAX: u16 = 0b11111;

#[derive(Copy, Clone, Debug)]
pub struct Bgr16(pub u16);

#[derive(Copy, Clone)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<Rgba> for Bgr16 {
    fn from(color: Rgba) -> Self {
        let cmf = SNES_BGR_CHANNEL_MAX as f32;
        let r = (color.r * cmf).round() as u16;
        let g = (color.g * cmf).round() as u16;
        let b = (color.b * cmf).round() as u16;
        Bgr16((r << 0x0) | (g << 0x5) | (b << 0xA))
    }
}

impl From<Bgr16> for Rgba {
    fn from(color: Bgr16) -> Self {
        let cmf = SNES_BGR_CHANNEL_MAX as f32;
        Rgba {
            r: ((color.0 & 0b000000000011111) >> 0x0) as f32 / cmf,
            g: ((color.0 & 0b000001111100000) >> 0x5) as f32 / cmf,
            b: ((color.0 & 0b111110000000000) >> 0xA) as f32 / cmf,
            a: 1.,
        }
    }
}

impl Into<[f32; 4]> for Rgba {
    fn into(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}