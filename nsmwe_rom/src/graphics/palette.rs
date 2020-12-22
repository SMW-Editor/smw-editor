use crate::{
    addr::conversions::snes_to_pc,
    graphics::color::{
        Bgr16,
        BGR16_SIZE,
    },
    internal_header::MapMode,
    level::primary_header::PrimaryHeader,
};
use self::constants::*;

use nom::{
    count,
    do_parse,
    map,
    named,
    number::complete::le_u16,
    IResult,
};

use std::convert::TryInto;

mod constants {
    use crate::graphics::color::BGR16_SIZE;

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

    pub const PALETTE_BG_SIZE:       usize = 0x18;
    pub const PALETTE_FG_SIZE:       usize = 0x18;
    pub const PALETTE_SPRITE_SIZE:   usize = 0x18;
    pub const PALETTE_WTF_SIZE:      usize = 11 * 0x0C;
    pub const PALETTE_PLAYER_SIZE:   usize = 4 * 0x14;
    pub const PALETTE_LAYER3_SIZE:   usize = 0x20;
    pub const PALETTE_BERRY_SIZE:    usize = 3 * 0x0E;
    pub const PALETTE_ANIMATED_SIZE: usize = 8 * BGR16_SIZE;

    pub const PALETTE_CUSTOM_LENGTH:   usize = 16 * 16;
    pub const PALETTE_BG_LENGTH:       usize = PALETTE_BG_SIZE / BGR16_SIZE;
    pub const PALETTE_FG_LENGTH:       usize = PALETTE_FG_SIZE / BGR16_SIZE;
    pub const PALETTE_SPRITE_LENGTH:   usize = PALETTE_SPRITE_SIZE / BGR16_SIZE;
    pub const PALETTE_WTF_LENGTH:      usize = PALETTE_WTF_SIZE / BGR16_SIZE;
    pub const PALETTE_PLAYER_LENGTH:   usize = PALETTE_PLAYER_SIZE / BGR16_SIZE;
    pub const PALETTE_LAYER3_LENGTH:   usize = PALETTE_LAYER3_SIZE / BGR16_SIZE;
    pub const PALETTE_BERRY_LENGTH:    usize = PALETTE_BERRY_SIZE / BGR16_SIZE;
    pub const PALETTE_ANIMATED_LENGTH: usize = PALETTE_ANIMATED_SIZE / BGR16_SIZE;
}

// -------------------------------------------------------------------------------------------------

pub struct CustomPalette {
    _back_area_color: Bgr16,
    _colors: [Bgr16; PALETTE_CUSTOM_LENGTH],
}

pub struct VanillaPalette {
    _back_area_color: Bgr16,
    _bg:             [Bgr16; PALETTE_BG_LENGTH],
    _fg:             [Bgr16; PALETTE_FG_LENGTH],
    _sprite:         [Bgr16; PALETTE_SPRITE_LENGTH],
    _wtf:            [Bgr16; PALETTE_WTF_LENGTH],
    _players:        [Bgr16; PALETTE_PLAYER_LENGTH],
    _layer3:         [Bgr16; PALETTE_LAYER3_LENGTH],
    _berry:          [Bgr16; PALETTE_BERRY_LENGTH],
    _animated_color: [Bgr16; PALETTE_ANIMATED_LENGTH],
}

// -------------------------------------------------------------------------------------------------

named!(le_bgr16<Bgr16>, map!(le_u16, Bgr16));

impl CustomPalette {
    named!(parse<&[u8], Self>, do_parse!(
        back_area_color: le_bgr16 >>
        colors: count!(le_bgr16, PALETTE_CUSTOM_LENGTH) >>
        (CustomPalette {
            _back_area_color: back_area_color,
            _colors: colors.try_into().unwrap(),
        })
    ));
}

impl VanillaPalette {
    pub fn from_primary_level_header<'a>(
        rom_data: &'a [u8],
        header: &PrimaryHeader,
        map_mode: MapMode,
    ) -> IResult<&'a [u8], VanillaPalette>
    {
        let parse_colors = |pos, n| {
            let pos = snes_to_pc::decide(map_mode)(pos).unwrap();
            let input = &rom_data[pos..pos + (2 * n)];
            count!(input, le_bgr16, n)
        };

        let (_, back_area_color) = parse_colors(
            addr::BACK_AREA_COLORS + (BGR16_SIZE * header.back_area_color as usize), 1)?;
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
            _back_area_color: back_area_color[0],
            _bg:              bg      .try_into().unwrap(),
            _fg:              fg      .try_into().unwrap(),
            _sprite:          sprite  .try_into().unwrap(),
            _wtf:             wtf     .try_into().unwrap(),
            _players:         players .try_into().unwrap(),
            _layer3:          layer3  .try_into().unwrap(),
            _berry:           berry   .try_into().unwrap(),
            _animated_color:  animated.try_into().unwrap(),
        }))
    }
}