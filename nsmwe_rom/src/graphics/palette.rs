use crate::{
    addr::AddrPc,
    error::{LevelPaletteError, RomParseError},
    graphics::color::{
        Bgr555,
        BGR555_SIZE,
    },
    level::{
        Level,
        primary_header::PrimaryHeader,
    },
};

use self::constants::*;

use nom::{
    count,
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

pub mod constants {
    use crate::graphics::color::BGR555_SIZE;

    pub mod addr {
        use crate::addr::AddrSnes;
        pub const BACK_AREA_COLORS: AddrSnes = AddrSnes(0x00B0A0);
        pub const BG_PALETTES:      AddrSnes = AddrSnes(0x00B0B0);
        pub const FG_PALETTES:      AddrSnes = AddrSnes(0x00B190);
        pub const SPRITE_PALETTES:  AddrSnes = AddrSnes(0x00B318);
        pub const WTF_PALETTE:      AddrSnes = AddrSnes(0x00B250);
        pub const PLAYER_PALETTE:   AddrSnes = AddrSnes(0x00B2C8);
        pub const LAYER3_PALETTE:   AddrSnes = AddrSnes(0x00B170);
        pub const BERRY_PALETTE:    AddrSnes = AddrSnes(0x00B674);
        pub const ANIMATED_COLOR:   AddrSnes = AddrSnes(0x00B60C);
    }

    pub const PALETTE_BG_SIZE:       usize = 0x18;
    pub const PALETTE_FG_SIZE:       usize = 0x18;
    pub const PALETTE_SPRITE_SIZE:   usize = 0x18;
    pub const PALETTE_WTF_SIZE:      usize = BGR555_SIZE * ((0xD - 0x4 + 1) * (0x7 - 0x2 + 1));
    pub const PALETTE_PLAYER_SIZE:   usize = 4 * 0x14;
    pub const PALETTE_LAYER3_SIZE:   usize = 0x20;
    pub const PALETTE_BERRY_SIZE:    usize = 3 * 0x0E;
    pub const PALETTE_ANIMATED_SIZE: usize = 8 * BGR555_SIZE;

    pub const PALETTE_LENGTH:          usize = 16 * 16;
    pub const PALETTE_BG_LENGTH:       usize = PALETTE_BG_SIZE / BGR555_SIZE;
    pub const PALETTE_FG_LENGTH:       usize = PALETTE_FG_SIZE / BGR555_SIZE;
    pub const PALETTE_SPRITE_LENGTH:   usize = PALETTE_SPRITE_SIZE / BGR555_SIZE;
    pub const PALETTE_WTF_LENGTH:      usize = PALETTE_WTF_SIZE / BGR555_SIZE;
    pub const PALETTE_PLAYER_LENGTH:   usize = PALETTE_PLAYER_SIZE / BGR555_SIZE;
    pub const PALETTE_LAYER3_LENGTH:   usize = PALETTE_LAYER3_SIZE / BGR555_SIZE;
    pub const PALETTE_BERRY_LENGTH:    usize = PALETTE_BERRY_SIZE / BGR555_SIZE;
    pub const PALETTE_ANIMATED_LENGTH: usize = PALETTE_ANIMATED_SIZE / BGR555_SIZE;
}

// -------------------------------------------------------------------------------------------------

pub trait ColorPalette {
    fn set_colors(&mut self, subpalette: &[Bgr555],
                  rows: RangeInclusive<usize>, cols: RangeInclusive<usize>)
    {
        let n_cols = *cols.end() - *cols.start() + 1;
        for (idx, &color) in subpalette.iter().enumerate() {
            let row = *rows.start() + (idx / n_cols);
            let col = *cols.start() + (idx % n_cols);
            debug_assert!(rows.contains(&row), "Row {} not between {}-{}", row, *rows.start(), *rows.end());
            debug_assert!(cols.contains(&col), "Col {} not between {}-{}", col, *cols.start(), *cols.end());
            self.set_color_at(row, col, color);
        }
    }

    fn set_color_at(&mut self, row: usize, col: usize, color: Bgr555);
    fn get_color_at(&self, row: usize, col: usize) -> Option<&Bgr555>;
}

macro_rules! impl_color_palette {
    ($struct_name:ident {
        $([$rows:expr, $cols:expr] => $field_name:ident),+
        $(, _ => $fallback:ident)? $(,)?
    }) => {
        impl ColorPalette for $struct_name {
            fn set_color_at(&mut self, row: usize, col: usize, color: Bgr555) {
                assert!(row <= 0xF);
                assert!(col <= 0xF);
                $(
                    if $rows.contains(&row) && $cols.contains(&col) {
                        let ri = row - *$rows.start();
                        let ci = col - *$cols.start();
                        self.$field_name[(ri * $cols.count()) + ci] = color;
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
            fn get_color_at(&self, row: usize, col: usize) -> Option<&Bgr555> {
                if row > 0xF || col > 0xF {
                    None
                } else {
                    $(
                        if $rows.contains(&row) && $cols.contains(&col) {
                            let ri = row - *$rows.start();
                            let ci = col - *$cols.start();
                            return Some(&self.$field_name[(ri * $cols.count()) + ci]);
                        }
                    )+
                    $(return self.$fallback.get_color_at(row, col);)?
                    Some(&Bgr555(0))
                }
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct GlobalLevelColorPalette {
    pub wtf:      Box<[Bgr555]>,
    pub players:  Box<[Bgr555]>,
    pub layer3:   Box<[Bgr555]>,
    pub berry:    Box<[Bgr555]>,
    pub animated: Box<[Bgr555]>,
}

#[derive(Clone)]
pub struct LevelColorPalette {
    pub global_palette: Rc<GlobalLevelColorPalette>,
    pub back_area_color: Bgr555,
    pub background: Box<[Bgr555]>,
    pub foreground: Box<[Bgr555]>,
    pub sprite: Box<[Bgr555]>,
}

#[derive(Clone)]
pub struct LevelColorPaletteSet {
    pub back_area_colors: Vec<Bgr555>,
    pub bg_palettes:      Vec<Box<[Bgr555]>>,
    pub fg_palettes:      Vec<Box<[Bgr555]>>,
    pub sprite_palettes:  Vec<Box<[Bgr555]>>,
}

// -------------------------------------------------------------------------------------------------

named!(le_bgr16<Bgr555>, map!(le_u16, Bgr555));

impl GlobalLevelColorPalette {
    pub fn parse(rom_data: &[u8]) -> IResult<&[u8], GlobalLevelColorPalette> {
        let parse_colors = |pos, n| {
            let pos: usize = AddrPc::try_from(pos).unwrap().into();
            preceded!(rom_data, take!(pos), count!(le_bgr16, n))
        };

        let (_, wtf)      = parse_colors(addr::WTF_PALETTE,    PALETTE_WTF_LENGTH)?;
        let (_, players)  = parse_colors(addr::PLAYER_PALETTE, PALETTE_PLAYER_LENGTH)?;
        let (_, layer3)   = parse_colors(addr::LAYER3_PALETTE, PALETTE_LAYER3_LENGTH)?;
        let (_, berry)    = parse_colors(addr::BERRY_PALETTE,  PALETTE_BERRY_LENGTH)?;
        let (_, animated) = parse_colors(addr::ANIMATED_COLOR, PALETTE_ANIMATED_LENGTH)?;

        let mut palette = GlobalLevelColorPalette {
            wtf:      [Bgr555::default(); PALETTE_WTF_LENGTH].into(),
            players:  [Bgr555::default(); PALETTE_PLAYER_LENGTH].into(),
            layer3:   [Bgr555::default(); PALETTE_LAYER3_LENGTH].into(),
            berry:    [Bgr555::default(); PALETTE_BERRY_LENGTH].into(),
            animated: [Bgr555::default(); PALETTE_ANIMATED_LENGTH].into(),
        };

        palette.set_colors(&wtf,     0x4..=0xD, 0x2..=0x7);
        palette.players = players.try_into().unwrap(); // 0x8..=0x8, 0x6..=0xF
        palette.set_colors(&layer3,  0x0..=0x1, 0x8..=0xF);
        palette.set_colors(&berry,   0x2..=0x4, 0x9..=0xF);
        palette.set_colors(&berry,   0x9..=0xB, 0x9..=0xF);
        palette.animated = animated.try_into().unwrap(); // 0x6, 0x4

        Ok((rom_data, palette))
    }

    pub fn is_animated_color_at(&self, row: usize, col: usize) -> bool {
        (row == 0x6) && (col == 0x4)
    }
}

impl LevelColorPaletteSet {
    pub fn parse(rom_data: &[u8], levels: &[Level]) -> Result<LevelColorPaletteSet, RomParseError> {
        let mut lvl_num = 0;
        match Self::parse_impl(rom_data, levels, &mut lvl_num) {
            Ok((_, palette_set)) => Ok(palette_set),
            Err(_) => Err(RomParseError::PaletteSetLevel(lvl_num)),
        }
    }

    fn parse_impl<'a>(rom_data: &'a [u8], levels: &[Level], lvl_num: &mut usize)
        -> IResult<&'a [u8], LevelColorPaletteSet>
    {
        let parse_colors = |pos, n| {
            let pos: usize = AddrPc::try_from(pos).unwrap().into();
            preceded!(rom_data, take!(pos), count!(le_bgr16, n))
        };

        let mut palette_set = LevelColorPaletteSet {
            back_area_colors: Vec::with_capacity(8),
            bg_palettes:      Vec::with_capacity(8),
            fg_palettes:      Vec::with_capacity(8),
            sprite_palettes:  Vec::with_capacity(8),
        };

        *lvl_num = 0;
        for level in levels {
            let header = &level.primary_header;

            let idx_bc = header.back_area_color as usize;
            let idx_bg = header.palette_bg as usize;
            let idx_fg = header.palette_fg as usize;
            let idx_sp = header.palette_sprite as usize;

            let (_, bc) = parse_colors(
                addr::BACK_AREA_COLORS + (BGR555_SIZE * idx_bc), 1)?;
            let (_, bg) = parse_colors(
                addr::BG_PALETTES + (PALETTE_BG_SIZE * idx_bg), PALETTE_BG_LENGTH)?;
            let (_, fg) = parse_colors(
                addr::FG_PALETTES + (PALETTE_FG_SIZE * idx_fg), PALETTE_FG_LENGTH)?;
            let (_, sp) = parse_colors(
                addr::SPRITE_PALETTES + (PALETTE_SPRITE_SIZE * idx_sp), PALETTE_SPRITE_LENGTH)?;

            if palette_set.back_area_colors.len() < idx_bc + 1 {
                let value = Bgr555::default();
                palette_set.back_area_colors.resize(idx_bc + 1, value);
            }
            if palette_set.bg_palettes.len() < idx_bg + 1 {
                let value: Box<[Bgr555]> = [Bgr555::default(); PALETTE_BG_LENGTH].into();
                palette_set.bg_palettes.resize(idx_bg + 1, value);
            }
            if palette_set.fg_palettes.len() < idx_fg + 1 {
                let value: Box<[Bgr555]> = [Bgr555::default(); PALETTE_FG_LENGTH].into();
                palette_set.fg_palettes.resize(idx_fg + 1, value);
            }
            if palette_set.sprite_palettes.len() < idx_sp + 1 {
                let value: Box<[Bgr555]> = [Bgr555::default(); PALETTE_SPRITE_LENGTH].into();
                palette_set.sprite_palettes.resize(idx_sp + 1, value);
            }

            palette_set.back_area_colors[idx_bc] = bc[0];
            palette_set.bg_palettes[idx_bg] = bg.try_into().unwrap();
            palette_set.fg_palettes[idx_fg] = fg.try_into().unwrap();
            palette_set.sprite_palettes[idx_sp] = sp.try_into().unwrap();

            *lvl_num += 1;
        }

        Ok((rom_data, palette_set))
    }

    pub fn get_level_palette(&self, header: &PrimaryHeader, gp: &Rc<GlobalLevelColorPalette>)
        -> Result<LevelColorPalette, LevelPaletteError>
    {
        let i_bc = header.back_area_color as usize;
        let i_bg = header.palette_bg as usize;
        let i_fg = header.palette_fg as usize;
        let i_sp = header.palette_sprite as usize;

        Ok(LevelColorPalette {
            global_palette: Rc::clone(gp),
            back_area_color: self.back_area_colors.get(i_bc).cloned()
                .ok_or(LevelPaletteError::BackAreaColor)?,
            background: self.bg_palettes.get(i_bg).cloned()
                .ok_or(LevelPaletteError::Background)?,
            foreground: self.fg_palettes.get(i_fg).cloned()
                .ok_or(LevelPaletteError::Foreground)?,
            sprite: self.sprite_palettes.get(i_sp).cloned()
                .ok_or(LevelPaletteError::Sprite)?,
        })
    }
}

impl_color_palette!(GlobalLevelColorPalette {
    [0x8..=0x8, 0x6..=0xF] => players,
    [0x0..=0x1, 0x8..=0xF] => layer3,
    [0x2..=0x4, 0x9..=0xF] => berry,
    [0x9..=0xB, 0x9..=0xF] => berry,
    [0x4..=0xD, 0x2..=0x7] => wtf,
});

impl_color_palette!(LevelColorPalette {
    [0x0..=0x1, 0x2..=0x7] => background,
    [0x2..=0x3, 0x2..=0x7] => foreground,
    [0xE..=0xF, 0x2..=0x7] => sprite,
    _ => global_palette,
});
