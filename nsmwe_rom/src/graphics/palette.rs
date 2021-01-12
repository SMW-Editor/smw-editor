use crate::{
    addr::AddrPc,
    graphics::color::{
        Bgr16,
        BGR16_SIZE,
    },
    level::primary_header::PrimaryHeader,
};

use self::constants::*;

use nom::{
    count,
    do_parse,
    map,
    named,
    number::complete::le_u16,
    preceded,
    take,
    IResult,
};

use std::{
    convert::{TryInto, TryFrom},
    ops::RangeInclusive,
    rc::Rc,
};

mod constants {
    use crate::graphics::color::BGR16_SIZE;

    pub mod addr {
        use crate::addr::AddrSnes;
        pub const BACK_AREA_COLORS: AddrSnes = AddrSnes(0x00B0A0);
        pub const BG_PALETTES:      AddrSnes = AddrSnes(0x00B0B0);
        pub const FG_PALETTES:      AddrSnes = AddrSnes(0x00B190);
        pub const SPRITE_PALETTES:  AddrSnes = AddrSnes(0x00B318);
        pub const WTF_PALETTES:     AddrSnes = AddrSnes(0x00B250);
        pub const PLAYER_PALETTES:  AddrSnes = AddrSnes(0x00B2C8);
        pub const LAYER3_PALETTES:  AddrSnes = AddrSnes(0x00B170);
        pub const BERRY_PALETTES:   AddrSnes = AddrSnes(0x00B674);
        pub const ANIMATED_COLOR:   AddrSnes = AddrSnes(0x00B60C);
    }

    pub const PALETTE_BG_SIZE:       usize = 0x18;
    pub const PALETTE_FG_SIZE:       usize = 0x18;
    pub const PALETTE_SPRITE_SIZE:   usize = 0x18;
    pub const PALETTE_WTF_SIZE:      usize = 11 * 0x0C;
    pub const PALETTE_PLAYER_SIZE:   usize = 4 * 0x14;
    pub const PALETTE_LAYER3_SIZE:   usize = 0x20;
    pub const PALETTE_BERRY_SIZE:    usize = 3 * 0x0E;
    pub const PALETTE_ANIMATED_SIZE: usize = 8 * BGR16_SIZE;

    pub const PALETTE_LENGTH:          usize = 16 * 16;
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

pub trait ColorPalette {
    fn set_colors(&mut self, subpalette: &[Bgr16],
                  rows: RangeInclusive<usize>, cols: RangeInclusive<usize>)
    {
        for (idx, &color) in subpalette.iter().enumerate() {
            let row = *rows.start() + (idx / cols.clone().count());
            let col = *cols.start() + (idx % cols.clone().count());
            self.set_color_at(row, col, color);
        }
    }

    fn set_color_at(&mut self, row: usize, col: usize, color: Bgr16);
    fn get_color_at(&self, row: usize, col: usize) -> Option<&Bgr16>;
}

macro_rules! impl_color_palette {
    ($struct_name:ident {
        $([$rows:expr, $cols:expr] => $field_name:ident),+
        $(, _ => $fallback:ident)? $(,)?
    }) => {
        impl ColorPalette for $struct_name {
            fn set_color_at(&mut self, row: usize, col: usize, color: Bgr16) {
                assert!(row <= 0xF);
                assert!(col <= 0xF);
                $(
                    if $rows.contains(&row) && $cols.contains(&col) {
                        let ri = row - *$rows.start();
                        let ci = col - *$cols.start();
                        self.$field_name[(ci * $rows.count()) + ri] = color;
                        return;
                    }
                )+
                $(
                    if let Some(fb) = Rc::get_mut(&mut self.$fallback) {
                        fb.set_color_at(row, col, color);
                    }
                )?
            }

            #[allow(unreachable_code)]
            fn get_color_at(&self, row: usize, col: usize) -> Option<&Bgr16> {
                if row > 0xF || col > 0xF {
                    None
                } else {
                    $(
                        if $rows.contains(&row) && $cols.contains(&col) {
                            let ri = row - *$rows.start();
                            let ci = col - *$cols.start();
                            return Some(&self.$field_name[(ci * $rows.count()) + ri]);
                        }
                    )+
                    $(return self.$fallback.get_color_at(row, col);)?
                    Some(&Bgr16(0))
                }
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct CustomColorPalette {
    pub back_area_color: Bgr16,
    pub colors: [Bgr16; PALETTE_LENGTH],
}

#[derive(Clone)]
pub struct GlobalLevelColorPalette {
    pub wtf:      [Bgr16; PALETTE_WTF_LENGTH],
    pub players:  [Bgr16; PALETTE_PLAYER_LENGTH],
    pub layer3:   [Bgr16; PALETTE_LAYER3_LENGTH],
    pub berry:    [Bgr16; PALETTE_BERRY_LENGTH],
    pub animated: [Bgr16; PALETTE_ANIMATED_LENGTH],
}

#[derive(Clone)]
pub struct LevelColorPalette {
    pub global_palette: Rc<GlobalLevelColorPalette>,
    pub back_area_color: Bgr16,
    pub bg:     [Bgr16; PALETTE_BG_LENGTH],
    pub fg:     [Bgr16; PALETTE_FG_LENGTH],
    pub sprite: [Bgr16; PALETTE_SPRITE_LENGTH],
}

// -------------------------------------------------------------------------------------------------

named!(le_bgr16<Bgr16>, map!(le_u16, Bgr16));

impl CustomColorPalette {
    // pub fn from_level_header<'a>(rom_data: &'a [u8], level_num: usize, header: &PrimaryHeader)
    //     -> IResult<&'a [u8], CustomColorPalette>
    // {
    //     CustomColorPalette::parse_vanilla_level_palette(rom_data, header);
    //     let palette_addr = LEVEL_PALETTES + (3 * level_num);
    //     if palette_addr == AddrSnes(0x0) || palette_addr == AddrSnes(0xFFFFFF) {
    //         CustomColorPalette::parse_vanilla_level_palette(rom_data, header)
    //     } else {
    //         CustomColorPalette::parse(rom_data)
    //     }
    // }

    named!(pub parse<&[u8], Self>, do_parse!(
        back_area_color: le_bgr16 >>
        colors: count!(le_bgr16, PALETTE_LENGTH) >>
        (CustomColorPalette {
            back_area_color: back_area_color,
            colors: colors.try_into().unwrap(),
        })
    ));

    fn get_index_at(row: usize, col: usize) -> usize {
        (row * 16) + col
    }
}

impl ColorPalette for CustomColorPalette {
    fn set_color_at(&mut self, x: usize, y: usize, col: Bgr16) {
        self.colors[Self::get_index_at(x, y)] = col;
    }

    fn get_color_at(&self, x: usize, y: usize) -> Option<&Bgr16> {
        let index = Self::get_index_at(x, y);
        self.colors.get(index)
    }
}

impl GlobalLevelColorPalette {
    pub fn parse(rom_data: &[u8]) -> IResult<&[u8], GlobalLevelColorPalette> {
        let parse_colors = |pos, n| {
            let pos: usize = AddrPc::try_from(pos).unwrap().into();
            preceded!(rom_data, take!(pos), count!(le_bgr16, n))
        };

        let (_, wtf)      = parse_colors(addr::WTF_PALETTES,    PALETTE_WTF_LENGTH)?;
        let (_, players)  = parse_colors(addr::PLAYER_PALETTES, PALETTE_PLAYER_LENGTH)?;
        let (_, layer3)   = parse_colors(addr::LAYER3_PALETTES, PALETTE_LAYER3_LENGTH)?;
        let (_, berry)    = parse_colors(addr::BERRY_PALETTES,  PALETTE_BERRY_LENGTH)?;
        let (_, animated) = parse_colors(addr::ANIMATED_COLOR,  PALETTE_ANIMATED_LENGTH)?;

        let mut palette = GlobalLevelColorPalette {
            wtf:      [Bgr16(0); PALETTE_WTF_LENGTH],
            players:  [Bgr16(0); PALETTE_PLAYER_LENGTH],
            layer3:   [Bgr16(0); PALETTE_LAYER3_LENGTH],
            berry:    [Bgr16(0); PALETTE_BERRY_LENGTH],
            animated: [Bgr16(0); PALETTE_ANIMATED_LENGTH],
        };

        palette.set_colors(&wtf,     0x4..=0xD, 0x2..=0x7);
        palette.set_colors(&players, 0x8..=0x8, 0x6..=0xF);
        palette.set_colors(&layer3,  0x0..=0x1, 0x8..=0xF);
        palette.set_colors(&berry,   0x2..=0x4, 0x9..=0xF);
        palette.set_colors(&berry,   0x9..=0xB, 0x9..=0xF);
        palette.set_color_at(0x6, 0x4, animated[0]);

        Ok((rom_data, palette))
    }
}

impl_color_palette!(GlobalLevelColorPalette {
    [0x4..=0xD, 0x2..=0x7] => wtf,
    [0x8..=0x8, 0x6..=0xF] => players,
    [0x0..=0x1, 0x8..=0xF] => layer3,
    [0x2..=0x4, 0x9..=0xF] => berry,
    [0x9..=0xB, 0x9..=0xF] => berry,
});

impl LevelColorPalette {
    pub fn parse<'a>(rom_data: &'a [u8], header: &PrimaryHeader, gp: Rc<GlobalLevelColorPalette>)
        -> IResult<&'a [u8], LevelColorPalette>
    {
        let parse_colors = |pos, n| {
            let pos: usize = AddrPc::try_from(pos).unwrap().into();
            preceded!(rom_data, take!(pos), count!(le_bgr16, n))
        };

        let (_, back_area_color) = parse_colors(
            addr::BACK_AREA_COLORS + (BGR16_SIZE * header.back_area_color as usize), 1)?;
        let (_, bg) = parse_colors(
            addr::BG_PALETTES + (PALETTE_BG_SIZE * header.palette_bg as usize),
            PALETTE_BG_LENGTH)?;
        let (_, fg) = parse_colors(
            addr::FG_PALETTES + (PALETTE_FG_SIZE * header.palette_fg as usize),
            PALETTE_FG_LENGTH)?;
        let (_, sprite) = parse_colors(
            addr::SPRITE_PALETTES + (PALETTE_SPRITE_SIZE * header.palette_sprite as usize),
            PALETTE_SPRITE_LENGTH)?;

        let mut palette = LevelColorPalette {
            global_palette: gp,
            back_area_color: back_area_color[0],
            bg:     [Bgr16(0); PALETTE_BG_LENGTH],
            fg:     [Bgr16(0); PALETTE_FG_LENGTH],
            sprite: [Bgr16(0); PALETTE_SPRITE_LENGTH],
        };

        palette.set_colors(&bg,     0x0..=0x1, 0x2..=0x7);
        palette.set_colors(&fg,     0x2..=0x3, 0x2..=0x7);
        palette.set_colors(&sprite, 0xE..=0xF, 0x2..=0x7);

        Ok((rom_data, palette))
    }
}

impl_color_palette!(LevelColorPalette {
    [0x0..=0x1, 0x2..=0x7] => bg,
    [0x2..=0x3, 0x2..=0x7] => fg,
    [0xE..=0xF, 0x2..=0x7] => sprite,
    _ => global_palette,
});
