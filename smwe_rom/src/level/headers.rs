use std::convert::{TryFrom, TryInto};

use nom::{number::complete::le_u8, preceded, take, IResult};

use crate::addr::{AddrPc, AddrSnes};

pub const PRIMARY_HEADER_SIZE: usize = 5;
pub const SECONDARY_HEADER_SIZE: usize = 4;
pub const SPRITE_HEADER_SIZE: usize = 1;

#[derive(Clone)]
pub struct PrimaryHeader([u8; PRIMARY_HEADER_SIZE]);

#[derive(Clone)]
pub struct SecondaryHeader([u8; SECONDARY_HEADER_SIZE]);

#[derive(Clone)]
pub struct SpriteHeader(u8);

impl PrimaryHeader {
    pub fn read_from(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, bytes) = take!(input, PRIMARY_HEADER_SIZE)?;
        Ok((input, Self(bytes.try_into().unwrap())))
    }

    pub fn palette_bg(&self) -> u8 {
        // BBB----- -------- -------- -------- --------
        // palette_bg = BBB

        self.0[0] >> 5
    }

    pub fn level_length(&self) -> u8 {
        // ---LLLLL -------- -------- -------- --------
        // level_length = LLLLL

        self.0[0] & 0b11111
    }

    pub fn back_area_color(&self) -> u8 {
        // -------- CCC----- -------- -------- --------
        // back_area_color = CCC

        self.0[1] >> 5
    }

    pub fn level_mode(&self) -> u8 {
        // -------- ---OOOOO -------- -------- --------
        // level_mode = OOOOO

        self.0[1] & 0b11111
    }

    pub fn layer3_priority(&self) -> bool {
        // -------- -------- 3------- -------- --------
        // layer3_priority = 3

        (self.0[2] >> 7) != 0
    }

    pub fn music(&self) -> u8 {
        // -------- -------- -MMM---- -------- --------
        // music = MMM

        (self.0[2] >> 4) & 0b111
    }

    pub fn sprite_gfx(&self) -> u8 {
        // -------- -------- ----SSSS -------- --------
        // sprite_gfx = SSSS

        self.0[2] & 0b1111
    }

    pub fn timer(&self) -> u8 {
        // -------- -------- -------- TT------ --------
        // timer = TT

        self.0[3] >> 6
    }

    pub fn palette_sprite(&self) -> u8 {
        // -------- -------- -------- --PPP--- --------
        // palette_sprite = PPP

        (self.0[3] >> 3) & 0b111
    }

    pub fn palette_fg(&self) -> u8 {
        // -------- -------- -------- -----FFF --------
        // palette_fg = FFF

        self.0[3] & 0b111
    }

    pub fn item_memory(&self) -> u8 {
        // -------- -------- -------- -------- II------
        // item_memory = II

        self.0[4] >> 6
    }

    pub fn vertical_scroll(&self) -> u8 {
        // -------- -------- -------- -------- --VV----
        // vertical_scroll = VV

        (self.0[4] >> 4) & 0b11
    }

    pub fn fg_bg_gfx(&self) -> u8 {
        // -------- -------- -------- -------- ----ZZZZ
        // fg_bg_gfx = ZZZZ

        self.0[4] & 0b1111
    }
}

impl SecondaryHeader {
    pub fn read_from_rom(rom_data: &[u8], level_num: usize) -> IResult<&[u8], Self> {
        let take_byte = |addr| {
            let addr: usize = AddrPc::try_from(AddrSnes(addr)).unwrap().into();
            preceded!(rom_data, take!(addr + level_num), le_u8)
        };
        let bytes = [take_byte(0x05F000)?.1, take_byte(0x05F200)?.1, take_byte(0x05F400)?.1, take_byte(0x05F600)?.1];
        Ok((rom_data, Self(bytes)))
    }

    pub fn layer2_scroll(&self) -> u8 {
        (self.0[0] >> 4) & 0b1111
    }

    pub fn main_entrance_xy_pos(&self) -> (u8, u8) {
        ((self.0[1] >> 0) & 0b111, (self.0[0] >> 0) & 0b1111)
    }

    pub fn layer3(&self) -> u8 {
        (self.0[1] >> 6) & 0b11
    }

    pub fn main_entrance_mario_action(&self) -> u8 {
        (self.0[1] >> 3) & 0b111
    }

    pub fn midway_entrance_screen(&self) -> u8 {
        (self.0[2] >> 4) & 0b1111
    }

    pub fn fg_initial_pos(&self) -> u8 {
        (self.0[2] >> 2) & 0b11
    }

    pub fn bg_initial_pos(&self) -> u8 {
        (self.0[2] >> 0) & 0b11
    }

    pub fn no_yoshi_level(&self) -> bool {
        ((self.0[3] >> 7) & 0b1) != 0
    }

    pub fn unknown_vertical_pos_level(&self) -> bool {
        ((self.0[3] >> 6) & 0b1) != 0
    }

    pub fn vertical_level(&self) -> bool {
        ((self.0[3] >> 5) & 0b1) != 0
    }

    pub fn main_entrance_screen(&self) -> u8 {
        (self.0[3] >> 0) & 0b11111
    }
}

impl SpriteHeader {
    pub fn read_from(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, bytes) = take!(input, SPRITE_HEADER_SIZE)?;
        Ok((input, Self(bytes[0])))
    }

    pub fn sprite_buoyancy(&self) -> bool {
        ((self.0 >> 7) & 0b1) != 0
    }

    pub fn disable_layer2_interaction(&self) -> bool {
        ((self.0 >> 6) & 0b1) != 0
    }

    pub fn sprite_memory(&self) -> u8 {
        (self.0 >> 0) & 0b111111
    }
}
