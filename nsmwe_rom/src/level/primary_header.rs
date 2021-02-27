use nom::{
    IResult,
    take,
};

pub const PRIMARY_HEADER_SIZE: usize = 5;

#[derive(Clone)]
pub struct PrimaryHeader {
    pub palette_bg: u8,
    pub level_length: u8,
    pub back_area_color: u8,
    pub level_mode: u8,
    pub layer3_priority: bool,
    pub music: u8,
    pub sprite_gfx: u8,
    pub timer: u8,
    pub palette_sprite: u8,
    pub palette_fg: u8,
    pub item_memory: u8,
    pub vertical_scroll: u8,
    pub fg_bg_gfx: u8,
}

impl PrimaryHeader {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, bytes) = take!(input, PRIMARY_HEADER_SIZE)?;
        Ok((input, PrimaryHeader {
            palette_bg:      ((bytes[0] & 0b11100000) >> 5),
            level_length:    ((bytes[0] & 0b00011111) >> 0),
            back_area_color: ((bytes[1] & 0b11100000) >> 5),
            level_mode:      ((bytes[1] & 0b00011111) >> 0),
            layer3_priority: ((bytes[2] & 0b10000000) >> 7) != 0,
            music:           ((bytes[2] & 0b01110000) >> 4),
            sprite_gfx:      ((bytes[2] & 0b00001111) >> 0),
            timer:           ((bytes[3] & 0b11000000) >> 6),
            palette_sprite:  ((bytes[3] & 0b00111000) >> 3),
            palette_fg:      ((bytes[3] & 0b00000111) >> 0),
            item_memory:     ((bytes[4] & 0b11000000) >> 6),
            vertical_scroll: ((bytes[4] & 0b00110000) >> 4),
            fg_bg_gfx:       ((bytes[4] & 0b00001111) >> 0),
        }))
    }
}
