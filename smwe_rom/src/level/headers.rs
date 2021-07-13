use std::convert::{TryFrom, TryInto};

use nom::{bytes::complete::take, number::complete::le_u8, sequence::preceded, IResult};

use crate::{
    error::{ParseErr, SecondaryHeaderParseError},
    snes_utils::addr::{AddrPc, AddrSnes},
};

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
        let (input, bytes) = take(PRIMARY_HEADER_SIZE)(input)?;
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
        // -------- ---MMMMM -------- -------- --------
        // level_mode = MMMMM
        self.0[1] & 0b11111
    }

    pub fn layer3_priority(&self) -> bool {
        // -------- -------- P------- -------- --------
        // layer3_priority = P
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
        // -------- -------- -------- -------- ----GGGG
        // fg_bg_gfx = GGGG
        self.0[4] & 0b1111
    }
}

impl SecondaryHeader {
    pub fn read_from_rom(rom_data: &[u8], level_num: usize) -> Result<Self, SecondaryHeaderParseError> {
        let take_byte = |addr| {
            let addr: usize =
                AddrPc::try_from(AddrSnes(addr)).map_err(SecondaryHeaderParseError::AddressConversion)?.into();
            let mut read_byte = preceded(take(addr + level_num), le_u8);
            read_byte(rom_data).map_err(|_: ParseErr| SecondaryHeaderParseError::Read)
        };
        let bytes = [take_byte(0x05F000)?.1, take_byte(0x05F200)?.1, take_byte(0x05F400)?.1, take_byte(0x05F600)?.1];
        Ok(Self(bytes))
    }

    pub fn layer2_scroll(&self) -> u8 {
        // SSSS---- -------- -------- --------
        // layer2_scroll = SSSS
        (self.0[0] >> 4) & 0b1111
    }

    pub fn main_entrance_xy_pos(&self) -> (u8, u8) {
        // ----YYYY -----XXX -------- --------
        // main_entrance_xy_pos = (XXX, YYYY)
        let x = self.0[1] & 0b111;
        let y = self.0[0] & 0b1111;
        (x, y)
    }

    pub fn layer3(&self) -> u8 {
        // -------- LL------ -------- --------
        // layer3 = LL
        (self.0[1] >> 6) & 0b11
    }

    pub fn main_entrance_mario_action(&self) -> u8 {
        // -------- --AAA--- -------- --------
        // main_entrance_mario_action = AAA
        (self.0[1] >> 3) & 0b111
    }

    pub fn midway_entrance_screen(&self) -> u8 {
        // -------- -------- SSSS---- --------
        // midway_entrance_screen = SSSS
        (self.0[2] >> 4) & 0b1111
    }

    pub fn fg_initial_pos(&self) -> u8 {
        // -------- -------- ----FF-- --------
        // fg_initial_pos = FF
        (self.0[2] >> 2) & 0b11
    }

    pub fn bg_initial_pos(&self) -> u8 {
        // -------- -------- ------BB --------
        // bg_initial_pos = BB
        self.0[2] & 0b11
    }

    pub fn no_yoshi_level(&self) -> bool {
        // -------- -------- -------- Y-------
        // no_yoshi_level = Y
        (self.0[3] >> 7) != 0
    }

    pub fn unknown_vertical_pos_level(&self) -> bool {
        // -------- -------- -------- -U------
        // unknown_vertical_pos_level = U
        (self.0[3] & 0b01000000) != 0
    }

    pub fn vertical_level(&self) -> bool {
        // -------- -------- -------- --V-----
        // vertical_level = V
        (self.0[3] & 0b00100000) != 0
    }

    pub fn main_entrance_screen(&self) -> u8 {
        // -------- -------- -------- ---EEEEE
        // main_entrance_screen = EEEEE
        self.0[3] & 0b11111
    }
}

impl SpriteHeader {
    pub fn read_from(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, bytes) = take(SPRITE_HEADER_SIZE)(input)?;
        Ok((input, Self(bytes[0])))
    }

    pub fn sprite_buoyancy(&self) -> bool {
        // B-------
        // sprite_buoyancy = B
        (self.0 & 0b10000000) != 0
    }

    pub fn disable_layer2_interaction(&self) -> bool {
        // -L------
        // disable_layer2_interaction = L
        (self.0 & 0b01000000) != 0
    }

    pub fn sprite_memory(&self) -> u8 {
        // --MMMMMM
        // sprite_memory = MMMMMM
        self.0 & 0b00111111
    }
}
