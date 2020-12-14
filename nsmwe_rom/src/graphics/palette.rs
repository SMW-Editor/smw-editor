use crate::{
    graphics::color::Bgr16,
    level::header::PrimaryLevelHeader,
};
use self::constants::*;

use nom::{
    count,
    do_parse,
    named,
    number::complete::le_u16,
    IResult,
};

use std::convert::TryInto;

mod constants {
    pub mod addr {
        use crate::addr::AddressSnes;
        pub const BACK_AREA_COLORS: AddressSnes = 0x00B0A0;
        pub const BG_PALETTES:      AddressSnes = 0x00B0B0;
        pub const FG_PALETTES:      AddressSnes = 0x00B190;
        pub const SPRITE_PALETTES:  AddressSnes = 0x00B318;
        pub const WTF_PALETTES:     AddressSnes = 0x00B250;
        pub const PLAYER_PALETTES:  AddressSnes = 0x00B2C8;
        pub const LAYER3_PALETTES:  AddressSnes = 0x00B170;
        pub const BERRY_PALETTES:   AddressSnes = 0x00B674;
        pub const ANIMATED_COLOR:   AddressSnes = 0x00B60C;
    }

    pub const COLORS_IN_CUSTOM_PALETTE: usize = 16 * 16;
    pub const COLOR_SIZE: usize = 0x02;

    pub const PALETTE_BG_SIZE:       usize = 0x18;
    pub const PALETTE_FG_SIZE:       usize = 0x18;
    pub const PALETTE_SPRITE_SIZE:   usize = 0x18;
    pub const PALETTE_WTF_SIZE:      usize = 11 * 0x0C;
    pub const PALETTE_PLAYER_SIZE:   usize = 4 * 0x14;
    pub const PALETTE_LAYER3_SIZE:   usize = 0x20;
    pub const PALETTE_BERRY_SIZE:    usize = 3 * 0x0E;
    pub const PALETTE_ANIMATED_SIZE: usize = 8 * COLOR_SIZE;

    pub const PALETTE_BG_LENGTH:       usize = PALETTE_BG_SIZE / COLOR_SIZE;
    pub const PALETTE_FG_LENGTH:       usize = PALETTE_FG_SIZE / COLOR_SIZE;
    pub const PALETTE_SPRITE_LENGTH:   usize = PALETTE_SPRITE_SIZE / COLOR_SIZE;
    pub const PALETTE_WTF_LENGTH:      usize = PALETTE_WTF_SIZE / COLOR_SIZE;
    pub const PALETTE_PLAYER_LENGTH:   usize = PALETTE_PLAYER_SIZE / COLOR_SIZE;
    pub const PALETTE_LAYER3_LENGTH:   usize = PALETTE_LAYER3_SIZE / COLOR_SIZE;
    pub const PALETTE_BERRY_LENGTH:    usize = PALETTE_BERRY_SIZE / COLOR_SIZE;
    pub const PALETTE_ANIMATED_LENGTH: usize = PALETTE_ANIMATED_SIZE / COLOR_SIZE;

    pub const ERR_VEC_LENGTH_MISMATCH: &str = "Number of elements in Vec is incorrect.";
}

// -------------------------------------------------------------------------------------------------

pub struct CustomPalette {
    back_area_color: Bgr16,
    colors: [Bgr16; COLORS_IN_CUSTOM_PALETTE],
}

pub struct VanillaPalette {
    back_area_color: Bgr16,
    bg:             [Bgr16; PALETTE_BG_LENGTH],
    fg:             [Bgr16; PALETTE_FG_LENGTH],
    sprite:         [Bgr16; PALETTE_SPRITE_LENGTH],
    wtf:            [Bgr16; PALETTE_WTF_LENGTH],
    players:        [Bgr16; PALETTE_PLAYER_LENGTH],
    layer3:         [Bgr16; PALETTE_LAYER3_LENGTH],
    berry:          [Bgr16; PALETTE_BERRY_LENGTH],
    animated_color: [Bgr16; PALETTE_ANIMATED_LENGTH],
}

// -------------------------------------------------------------------------------------------------

impl CustomPalette {
    named!(parse<&[u8], Self>, do_parse!(
        back_area_color: le_u16 >>
        colors: count!(le_u16, COLORS_IN_CUSTOM_PALETTE) >>
        (CustomPalette {
            back_area_color,
            colors: colors.try_into().expect(ERR_VEC_LENGTH_MISMATCH)
        })
    ));
}

impl VanillaPalette {
    pub fn from_primary_level_header<'a>(rom_data: &'a [u8], header: &PrimaryLevelHeader)
        -> IResult<&'a [u8], VanillaPalette>
    {
        let parse_colors = |pos, n| count!(&rom_data[pos..pos + (2 * n)], le_u16, n);

        let (_, back_area_color) = {
            let addr = addr::BACK_AREA_COLORS + (COLOR_SIZE * header.back_area_color as usize);
            le_u16(&rom_data[addr..addr + COLOR_SIZE])?
        };

        let (_, bg) = parse_colors(
            addr::BG_PALETTES + (PALETTE_BG_SIZE * header.palette_bg as usize), PALETTE_BG_LENGTH)?;
        let (_, fg) = parse_colors(
            addr::FG_PALETTES + (PALETTE_FG_SIZE * header.palette_fg as usize), PALETTE_FG_LENGTH)?;
        let (_, sprite) = parse_colors(
            addr::SPRITE_PALETTES + (PALETTE_SPRITE_SIZE * header.palette_sprite as usize),
            PALETTE_SPRITE_LENGTH)?;

        let (_, wtf)      = parse_colors(addr::WTF_PALETTES,    PALETTE_WTF_LENGTH)?;
        let (_, players)  = parse_colors(addr::PLAYER_PALETTES, PALETTE_PLAYER_LENGTH)?;
        let (_, layer3)   = parse_colors(addr::LAYER3_PALETTES, PALETTE_LAYER3_LENGTH)?;
        let (_, berry)    = parse_colors(addr::BERRY_PALETTES,  PALETTE_BERRY_LENGTH)?;
        let (_, animated) = parse_colors(addr::ANIMATED_COLOR,  PALETTE_ANIMATED_LENGTH)?;

        Ok((rom_data, VanillaPalette {
            back_area_color,
            bg:             bg      .try_into().expect(ERR_VEC_LENGTH_MISMATCH),
            fg:             fg      .try_into().expect(ERR_VEC_LENGTH_MISMATCH),
            sprite:         sprite  .try_into().expect(ERR_VEC_LENGTH_MISMATCH),
            wtf:            wtf     .try_into().expect(ERR_VEC_LENGTH_MISMATCH),
            players:        players .try_into().expect(ERR_VEC_LENGTH_MISMATCH),
            layer3:         layer3  .try_into().expect(ERR_VEC_LENGTH_MISMATCH),
            berry:          berry   .try_into().expect(ERR_VEC_LENGTH_MISMATCH),
            animated_color: animated.try_into().expect(ERR_VEC_LENGTH_MISMATCH),
        }))
    }
}