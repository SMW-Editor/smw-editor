use std::convert::TryFrom;

use nom::{number::complete::le_u8, preceded, take, IResult};

use crate::addr::{AddrPc, AddrSnes};

pub const PRIMARY_HEADER_SIZE: usize = 5;

#[derive(Clone)]
pub struct PrimaryHeader {
    pub palette_bg:      u8,
    pub level_length:    u8,
    pub back_area_color: u8,
    pub level_mode:      u8,
    pub layer3_priority: bool,
    pub music:           u8,
    pub sprite_gfx:      u8,
    pub timer:           u8,
    pub palette_sprite:  u8,
    pub palette_fg:      u8,
    pub item_memory:     u8,
    pub vertical_scroll: u8,
    pub fg_bg_gfx:       u8,
}

#[derive(Clone)]
pub struct SecondaryHeader {
    pub layer2_scroll:              u8,
    pub main_entrance_pos:          (u8, u8),
    pub layer3:                     u8,
    pub main_entrance_mario_action: u8,
    pub midway_entrance_screen:     u8,
    pub fg_initial_pos:             u8,
    pub bg_initial_pos:             u8,
    pub no_yoshi_level:             bool,
    pub unknown_vertical_pos_level: bool,
    pub vertical_level:             bool,
    pub main_entrance_screen:       u8,
}

impl PrimaryHeader {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, bytes) = take!(input, PRIMARY_HEADER_SIZE)?;
        Ok((input, Self {
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

impl SecondaryHeader {
    pub fn parse(rom_data: &[u8], level_number: usize) -> IResult<&[u8], Self> {
        let take_byte = |addr| {
            let addr: usize = AddrPc::try_from(AddrSnes(addr)).unwrap().into();
            preceded!(rom_data, take!(addr + level_number), le_u8)
        };
        let bytes = [take_byte(0x05F000)?.1, take_byte(0x05F200)?.1, take_byte(0x05F400)?.1, take_byte(0x05F600)?.1];
        Ok((rom_data, Self {
            layer2_scroll:              (bytes[0] >> 4),
            main_entrance_pos:          (bytes[1] & 0b111, bytes[0] & 0b1111),
            layer3:                     (bytes[1] >> 6),
            main_entrance_mario_action: (bytes[1] >> 3) & 0b111,
            midway_entrance_screen:     (bytes[2] >> 4),
            fg_initial_pos:             (bytes[2] >> 2) & 0b11,
            bg_initial_pos:             (bytes[2] & 0b11),
            no_yoshi_level:             (bytes[3] >> 7) != 0,
            unknown_vertical_pos_level: ((bytes[3] >> 6) & 1) != 0,
            vertical_level:             ((bytes[3] >> 5) & 1) != 0,
            main_entrance_screen:       (bytes[3] & 0b11111),
        }))
    }
}
