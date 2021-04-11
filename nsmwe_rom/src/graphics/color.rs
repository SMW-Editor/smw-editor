#![allow(clippy::unusual_byte_groupings)]
#![allow(dead_code)]

pub const BGR555_SIZE: usize = std::mem::size_of::<Abgr1555>();

const SNES_BGR_CHANNEL_MAX: u16 = 0b11111;

// -------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug)]
pub struct Abgr1555(pub u16);

#[derive(Copy, Clone)]
pub struct Rgba32 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

// -------------------------------------------------------------------------------------------------

impl Abgr1555 {
    pub const TRANSPARENT: Abgr1555 = Abgr1555(0b1_00000_00000_00000);
    pub const BLACK:       Abgr1555 = Abgr1555(0b0_00000_00000_00000);
    pub const WHITE:       Abgr1555 = Abgr1555(0b0_11111_11111_11111);
    pub const RED:         Abgr1555 = Abgr1555(0b0_00000_00000_11111);
    pub const GREEN:       Abgr1555 = Abgr1555(0b0_00000_11111_00000);
    pub const BLUE:        Abgr1555 = Abgr1555(0b0_11111_00000_00000);
    pub const MAGENTA:     Abgr1555 = Abgr1555(0b0_11111_00000_11111);
}

impl From<Rgba32> for Abgr1555 {
    fn from(color: Rgba32) -> Self {
        let cmf = SNES_BGR_CHANNEL_MAX as f32;
        let r = (color.r * cmf).round() as u16 & SNES_BGR_CHANNEL_MAX;
        let g = (color.g * cmf).round() as u16 & SNES_BGR_CHANNEL_MAX;
        let b = (color.b * cmf).round() as u16 & SNES_BGR_CHANNEL_MAX;
        let a = (color.a as u16) & 1;
        Abgr1555(((1 - a) << 0xF) | (b << 0xA) | (g << 0x5) | (r << 0x0))
    }
}

impl Default for Abgr1555 {
    fn default() -> Self {
        Abgr1555::TRANSPARENT
    }
}

impl Rgba32 {
    pub const TRANSPARENT: Rgba32 = Rgba32::new(0.0, 0.0, 0.0, 0.0);
    pub const BLACK:       Rgba32 = Rgba32::new(0.0, 0.0, 0.0, 1.0);
    pub const WHITE:       Rgba32 = Rgba32::new(1.0, 1.0, 1.0, 1.0);
    pub const RED:         Rgba32 = Rgba32::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN:       Rgba32 = Rgba32::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE:        Rgba32 = Rgba32::new(0.0, 0.0, 1.0, 1.0);

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Rgba32 { r, g, b, a }
    }

    pub const fn as_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub const fn as_tuple(&self) -> (f32, f32, f32, f32) {
        (self.r, self.g, self.b, self.a)
    }
}

impl From<Abgr1555> for Rgba32 {
    fn from(color: Abgr1555) -> Self {
        let cmf = SNES_BGR_CHANNEL_MAX as f32;
        Rgba32 {
            r: ((color.0 >> 0x0) & SNES_BGR_CHANNEL_MAX) as f32 / cmf,
            g: ((color.0 >> 0x5) & SNES_BGR_CHANNEL_MAX) as f32 / cmf,
            b: ((color.0 >> 0xA) & SNES_BGR_CHANNEL_MAX) as f32 / cmf,
            a: 1.0 - ((color.0 >> 0xF) & 1) as f32,
        }
    }
}

impl From<[f32; 4]> for Rgba32 {
    fn from(color: [f32; 4]) -> Self {
        Rgba32 {
            r: color[0],
            g: color[1],
            b: color[2],
            a: color[3],
        }
    }
}

impl From<Rgba32> for [f32; 4] {
    fn from(this: Rgba32) -> [f32; 4] {
        this.as_array()
    }
}

impl From<(f32, f32, f32, f32)> for Rgba32 {
    fn from(color: (f32, f32, f32, f32)) -> Self {
        Rgba32 {
            r: color.0,
            g: color.1,
            b: color.2,
            a: color.3,
        }
    }
}

impl From<Rgba32> for (f32, f32, f32, f32) {
    fn from(this: Rgba32) -> (f32, f32, f32, f32) {
        this.as_tuple()
    }
}

impl Default for Rgba32 {
    fn default() -> Self {
        Rgba32::TRANSPARENT
    }
}
