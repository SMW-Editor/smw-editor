pub type Bgr16 = u16;
pub type Rgba = [f32; 4];

const SNES_BGR_CHANNEL_MAX: u16 = 0b11111;

pub fn bgr16_to_rgba(color: Bgr16) -> Rgba {
    let cmf = SNES_BGR_CHANNEL_MAX as f32;
    [
        ((color & 0b000000000011111) >> 0x0) as f32 / cmf,
        ((color & 0b000001111100000) >> 0x5) as f32 / cmf,
        ((color & 0b111110000000000) >> 0xA) as f32 / cmf,
        1.,
    ]
}

pub fn rgba_to_bgr16(color: Rgba) -> Bgr16 {
    let cmf = SNES_BGR_CHANNEL_MAX as f32;
    let r = (color[0] * cmf).round() as u16;
    let g = (color[1] * cmf).round() as u16;
    let b = (color[2] * cmf).round() as u16;
    (r << 0x0) | (g << 0x5) | (b << 0xA)
}