pub const BGR555_SIZE: usize = std::mem::size_of::<Bgr555>();

const SNES_BGR_CHANNEL_MAX: u16 = 0b11111;

#[derive(Copy, Clone, Debug)]
pub struct Bgr555(pub u16);

#[derive(Copy, Clone)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<Rgba> for Bgr555 {
    fn from(color: Rgba) -> Self {
        let cmf = SNES_BGR_CHANNEL_MAX as f32;
        let r = (color.r * cmf).round() as u16;
        let g = (color.g * cmf).round() as u16;
        let b = (color.b * cmf).round() as u16;
        Bgr555((r << 0x0) | (g << 0x5) | (b << 0xA))
    }
}

impl Default for Bgr555 {
    fn default() -> Self {
        Bgr555(0)
    }
}

impl From<Bgr555> for Rgba {
    fn from(color: Bgr555) -> Self {
        let cmf = SNES_BGR_CHANNEL_MAX as f32;
        Rgba {
            r: ((color.0 & 0b000000000011111) >> 0x0) as f32 / cmf,
            g: ((color.0 & 0b000001111100000) >> 0x5) as f32 / cmf,
            b: ((color.0 & 0b111110000000000) >> 0xA) as f32 / cmf,
            a: 1.,
        }
    }
}

impl From<[f32; 4]> for Rgba {
    fn from(color: [f32; 4]) -> Self {
        Rgba {
            r: color[0],
            g: color[1],
            b: color[2],
            a: color[3],
        }
    }
}

impl Into<[f32; 4]> for Rgba {
    fn into(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl Default for Rgba {
    fn default() -> Self {
        Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
}
