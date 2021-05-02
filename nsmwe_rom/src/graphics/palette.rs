use std::{convert::TryInto, ops::RangeInclusive, rc::Rc};

use nom::{
    count,
    map,
    named,
    number::complete::{le_u16, le_u8},
    preceded,
    take,
    IResult,
};

use self::constants::*;
use crate::{
    addr::{AddrPc, AddrSnes},
    error::{LevelPaletteError, OverworldSubmapPaletteError, RomParseError},
    graphics::color::{Abgr1555, BGR555_SIZE},
    level::{primary_header::PrimaryHeader, Level},
};

#[rustfmt::skip]
pub mod constants {
    use crate::graphics::color::BGR555_SIZE;

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

    pub const OW_LAYER1_PALETTE_LENGTH: usize = 7 * 6;
    pub const OW_LAYER2_PALETTE_LENGTH: usize = 7 * 4;
    pub const OW_LAYER3_PALETTE_LENGTH: usize = 8 * 2;
    pub const OW_SPRITE_PALETTE_LENGTH: usize = 7 * 7;
}

// -------------------------------------------------------------------------------------------------

pub trait ColorPalette {
    fn set_colors(
        &mut self, //
        subpalette: &[Abgr1555],
        rows: RangeInclusive<usize>,
        cols: RangeInclusive<usize>,
    ) {
        let n_cols = *cols.end() - *cols.start() + 1;
        for (idx, &color) in subpalette.iter().enumerate() {
            let row = *rows.start() + (idx / n_cols);
            let col = *cols.start() + (idx % n_cols);
            debug_assert!(rows.contains(&row), "Row {} not between {}-{}", row, *rows.start(), *rows.end());
            debug_assert!(cols.contains(&col), "Col {} not between {}-{}", col, *cols.start(), *cols.end());
            self.set_color_at(row, col, color);
        }
    }

    fn get_row(&self, r: usize) -> [Abgr1555; 16] {
        let mut row = [Abgr1555::TRANSPARENT; 16];
        for (c, color) in row.iter_mut().enumerate() {
            *color = self.get_color_at(r, c).unwrap_or(Abgr1555::MAGENTA);
        }
        row
    }

    fn set_color_at(&mut self, row: usize, col: usize, color: Abgr1555);
    fn get_color_at(&self, row: usize, col: usize) -> Option<Abgr1555>;
}

macro_rules! impl_color_palette {
    ($struct_name:ident {
        $([$rows:expr, $cols:expr] => $field_name:ident),+
        $(, _ => $fallback:ident)? $(,)?
    }) => {
        impl ColorPalette for $struct_name {
            fn set_color_at(&mut self, row: usize, col: usize, color: Abgr1555) {
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
            fn get_color_at(&self, row: usize, col: usize) -> Option<Abgr1555> {
                if row > 0xF || col > 0xF {
                    None
                } else if col == 0 {
                    Some(Abgr1555::TRANSPARENT)
                } else if col == 1 {
                    Some(Abgr1555::WHITE)
                } else {
                    $(
                        if $rows.contains(&row) && $cols.contains(&col) {
                            let ri = row - *$rows.start();
                            let ci = col - *$cols.start();
                            return Some(self.$field_name[(ri * $cols.count()) + ci]);
                        }
                    )+
                    $(return self.$fallback.get_color_at(row, col);)?
                    Some(Abgr1555::TRANSPARENT)
                }
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct GlobalLevelColorPalette {
    pub wtf:      Box<[Abgr1555]>,
    pub players:  Box<[Abgr1555]>,
    pub layer3:   Box<[Abgr1555]>,
    pub berry:    Box<[Abgr1555]>,
    pub animated: Box<[Abgr1555]>,
}

#[derive(Clone)]
pub struct LevelColorPalette {
    pub global_palette:  Rc<GlobalLevelColorPalette>,
    pub back_area_color: Abgr1555,
    pub background:      Box<[Abgr1555]>,
    pub foreground:      Box<[Abgr1555]>,
    pub sprite:          Box<[Abgr1555]>,
}

#[derive(Clone)]
pub struct LevelColorPaletteSet {
    pub back_area_colors: Vec<Abgr1555>,
    pub bg_palettes:      Vec<Box<[Abgr1555]>>,
    pub fg_palettes:      Vec<Box<[Abgr1555]>>,
    pub sprite_palettes:  Vec<Box<[Abgr1555]>>,
}

#[derive(Clone)]
pub struct GlobalOverworldColorPalette {
    pub layer1: Box<[Abgr1555]>,
    pub layer3: Box<[Abgr1555]>,
    pub sprite: Box<[Abgr1555]>,
}

#[derive(Clone)]
pub struct OverworldSubmapColorPalette {
    pub global_palette: Rc<GlobalOverworldColorPalette>,
    pub layer2:         Box<[Abgr1555]>,
}

#[derive(Clone)]
pub struct OverworldColorPaletteSet {
    pub layer2_pre_special:  Vec<Box<[Abgr1555]>>,
    pub layer2_post_special: Vec<Box<[Abgr1555]>>,
    pub layer2_indices:      Vec<usize>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum OverworldState {
    PreSpecial,
    PostSpecial,
}

// -------------------------------------------------------------------------------------------------

named!(le_bgr16<Abgr1555>, map!(le_u16, Abgr1555));

type ColorParserResult<'r> = IResult<&'r [u8], Vec<Abgr1555>>;

fn make_color_parser<'r>(rom_data: &'r [u8]) -> impl Fn(AddrSnes, usize) -> ColorParserResult<'r> + 'r {
    move |pos, n| {
        let pos: AddrPc = pos.try_into().unwrap();
        preceded!(rom_data, take!(pos.0), count!(le_bgr16, n))
    }
}

impl GlobalLevelColorPalette {
    pub fn parse(rom_data: &[u8]) -> Result<GlobalLevelColorPalette, RomParseError> {
        match Self::parse_impl(rom_data) {
            Ok((_, palette_set)) => Ok(palette_set),
            Err(_) => Err(RomParseError::PaletteGlobalLevel),
        }
    }

    fn parse_impl(rom_data: &[u8]) -> IResult<&[u8], GlobalLevelColorPalette> {
        const WTF_PALETTE: AddrSnes = AddrSnes(0x00B250);
        const PLAYER_PALETTE: AddrSnes = AddrSnes(0x00B2C8);
        const LAYER3_PALETTE: AddrSnes = AddrSnes(0x00B170);
        const BERRY_PALETTE: AddrSnes = AddrSnes(0x00B674);
        const ANIMATED_COLOR: AddrSnes = AddrSnes(0x00B60C);

        let parse_colors = make_color_parser(rom_data);

        let (_, wtf) = parse_colors(WTF_PALETTE, PALETTE_WTF_LENGTH)?;
        let (_, players) = parse_colors(PLAYER_PALETTE, PALETTE_PLAYER_LENGTH)?;
        let (_, layer3) = parse_colors(LAYER3_PALETTE, PALETTE_LAYER3_LENGTH)?;
        let (_, berry) = parse_colors(BERRY_PALETTE, PALETTE_BERRY_LENGTH)?;
        let (_, animated) = parse_colors(ANIMATED_COLOR, PALETTE_ANIMATED_LENGTH)?;

        let mut palette = GlobalLevelColorPalette {
            wtf:      [Abgr1555::default(); PALETTE_WTF_LENGTH].into(),
            players:  [Abgr1555::default(); PALETTE_PLAYER_LENGTH].into(),
            layer3:   [Abgr1555::default(); PALETTE_LAYER3_LENGTH].into(),
            berry:    [Abgr1555::default(); PALETTE_BERRY_LENGTH].into(),
            animated: [Abgr1555::default(); PALETTE_ANIMATED_LENGTH].into(),
        };

        palette.set_colors(&wtf, 0x4..=0xD, 0x2..=0x7);
        palette.players = players.try_into().unwrap(); // 0x8..=0x8, 0x6..=0xF
        palette.set_colors(&layer3, 0x0..=0x1, 0x8..=0xF);
        palette.set_colors(&berry, 0x2..=0x4, 0x9..=0xF);
        palette.set_colors(&berry, 0x9..=0xB, 0x9..=0xF);
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

    fn parse_impl<'a>(
        rom_data: &'a [u8], levels: &[Level], lvl_num: &mut usize,
    ) -> IResult<&'a [u8], LevelColorPaletteSet> {
        const BACK_AREA_COLORS: AddrSnes = AddrSnes(0x00B0A0);
        const BG_PALETTES: AddrSnes = AddrSnes(0x00B0B0);
        const FG_PALETTES: AddrSnes = AddrSnes(0x00B190);
        const SPRITE_PALETTES: AddrSnes = AddrSnes(0x00B318);

        let parse_colors = make_color_parser(rom_data);

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

            let (_, bc) = parse_colors(BACK_AREA_COLORS + (BGR555_SIZE * idx_bc), 1)?;
            let (_, bg) = parse_colors(BG_PALETTES + (PALETTE_BG_SIZE * idx_bg), PALETTE_BG_LENGTH)?;
            let (_, fg) = parse_colors(FG_PALETTES + (PALETTE_FG_SIZE * idx_fg), PALETTE_FG_LENGTH)?;
            let (_, sp) = parse_colors(SPRITE_PALETTES + (PALETTE_SPRITE_SIZE * idx_sp), PALETTE_SPRITE_LENGTH)?;

            if palette_set.back_area_colors.len() < idx_bc + 1 {
                let value = Abgr1555::default();
                palette_set.back_area_colors.resize(idx_bc + 1, value);
            }
            if palette_set.bg_palettes.len() < idx_bg + 1 {
                let value: Box<[Abgr1555]> = [Abgr1555::default(); PALETTE_BG_LENGTH].into();
                palette_set.bg_palettes.resize(idx_bg + 1, value);
            }
            if palette_set.fg_palettes.len() < idx_fg + 1 {
                let value: Box<[Abgr1555]> = [Abgr1555::default(); PALETTE_FG_LENGTH].into();
                palette_set.fg_palettes.resize(idx_fg + 1, value);
            }
            if palette_set.sprite_palettes.len() < idx_sp + 1 {
                let value: Box<[Abgr1555]> = [Abgr1555::default(); PALETTE_SPRITE_LENGTH].into();
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

    pub fn get_level_palette(
        &self, header: &PrimaryHeader, gp: &Rc<GlobalLevelColorPalette>,
    ) -> Result<LevelColorPalette, LevelPaletteError> {
        let i_back_area_color = header.back_area_color as usize;
        let i_background = header.palette_bg as usize;
        let i_foreground = header.palette_fg as usize;
        let i_sprite = header.palette_sprite as usize;
        self.palette_from_indices(i_back_area_color, i_background, i_foreground, i_sprite, gp)
    }

    pub fn palette_from_indices(
        &self, i_back_area_color: usize, i_background: usize, i_foreground: usize, i_sprite: usize,
        gp: &Rc<GlobalLevelColorPalette>,
    ) -> Result<LevelColorPalette, LevelPaletteError> {
        Ok(LevelColorPalette {
            global_palette:  Rc::clone(gp),
            back_area_color: self
                .back_area_colors
                .get(i_back_area_color)
                .cloned()
                .ok_or(LevelPaletteError::BackAreaColor)?,
            background:      self.bg_palettes.get(i_background).cloned().ok_or(LevelPaletteError::Background)?,
            foreground:      self.fg_palettes.get(i_foreground).cloned().ok_or(LevelPaletteError::Foreground)?,
            sprite:          self.sprite_palettes.get(i_sprite).cloned().ok_or(LevelPaletteError::Sprite)?,
        })
    }
}

impl GlobalOverworldColorPalette {
    pub fn parse(rom_data: &[u8]) -> Result<GlobalOverworldColorPalette, RomParseError> {
        match Self::parse_impl(rom_data) {
            Ok((_, palette_set)) => Ok(palette_set),
            Err(_) => Err(RomParseError::PaletteGlobalOverworld),
        }
    }

    fn parse_impl(rom_data: &[u8]) -> IResult<&[u8], GlobalOverworldColorPalette> {
        let parse_colors = make_color_parser(rom_data);

        const LAYER1_PALETTES: AddrSnes = AddrSnes(0x00B528);
        const LAYER3_PALETTES: AddrSnes = AddrSnes(0x00B5EC);
        const SPRITE_PALETTES: AddrSnes = AddrSnes(0x00B58A);

        let (_, layer1_colors) = parse_colors(LAYER1_PALETTES, OW_LAYER1_PALETTE_LENGTH)?;
        let (_, layer3_colors) = parse_colors(LAYER3_PALETTES, OW_LAYER3_PALETTE_LENGTH)?;
        let (_, sprite_colors) = parse_colors(SPRITE_PALETTES, OW_SPRITE_PALETTE_LENGTH)?;

        Ok((rom_data, GlobalOverworldColorPalette {
            layer1: layer1_colors.into(),
            layer3: layer3_colors.into(),
            sprite: sprite_colors.into(),
        }))
    }
}

impl OverworldColorPaletteSet {
    pub fn parse(rom_data: &[u8]) -> Result<OverworldColorPaletteSet, RomParseError> {
        match Self::parse_impl(rom_data) {
            Ok((_, palette_set)) => Ok(palette_set),
            Err(_) => Err(RomParseError::PaletteSetOverworld),
        }
    }

    fn parse_impl(rom_data: &[u8]) -> IResult<&[u8], OverworldColorPaletteSet> {
        let parse_colors = make_color_parser(rom_data);

        const LAYER2_NORMAL_PALETTES: AddrSnes = AddrSnes(0x00B3D8);
        const LAYER2_SPECIAL_PALETTES: AddrSnes = AddrSnes(0x00B732);
        const LAYER2_PALETTE_INDIRECT1: AddrSnes = AddrSnes(0x00AD1E);
        const LAYER2_PALETTE_INDIRECT2: AddrSnes = AddrSnes(0x00ABDF);

        let mut layer2_pre_special = Vec::with_capacity(6);
        let mut layer2_post_special = Vec::with_capacity(6);
        let mut layer2_indices = Vec::with_capacity(7);

        for i in 0..6 {
            let subworld_pal_idx = i * 14 * 4;
            let (_, layer2_colors_normal) =
                parse_colors(LAYER2_NORMAL_PALETTES + subworld_pal_idx, OW_LAYER2_PALETTE_LENGTH)?;
            let (_, layer2_colors_special) =
                parse_colors(LAYER2_SPECIAL_PALETTES + subworld_pal_idx, OW_LAYER2_PALETTE_LENGTH)?;

            layer2_pre_special.push(layer2_colors_normal.into());
            layer2_post_special.push(layer2_colors_special.into());
        }

        let indirect1_addr: AddrPc = LAYER2_PALETTE_INDIRECT1.try_into().unwrap();
        let indirect2_addr: AddrPc = LAYER2_PALETTE_INDIRECT2.try_into().unwrap();
        let (_, indirect1) = preceded!(rom_data, take!(indirect1_addr.0), count!(le_u8, 7))?;
        for offset in indirect1.into_iter() {
            let (_, ptr16) = preceded!(rom_data, take!(indirect2_addr.0 + (2 * offset as usize)), le_u16)?;
            let idx = ptr16 / 0x38;
            layer2_indices.push(idx as usize);
        }

        Ok((rom_data, OverworldColorPaletteSet { layer2_pre_special, layer2_post_special, layer2_indices }))
    }

    pub fn get_submap_palette(
        &self, submap: usize, ow_state: OverworldState, gp: &Rc<GlobalOverworldColorPalette>,
    ) -> Result<OverworldSubmapColorPalette, OverworldSubmapPaletteError> {
        let i_submap_palette = *self.layer2_indices.get(submap).ok_or(OverworldSubmapPaletteError::Layer2)?;
        self.get_submap_palette_from_indices(i_submap_palette, ow_state, gp)
    }

    pub fn get_submap_palette_from_indices(
        &self, i_submap_palette: usize, ow_state: OverworldState, gp: &Rc<GlobalOverworldColorPalette>,
    ) -> Result<OverworldSubmapColorPalette, OverworldSubmapPaletteError> {
        Ok(OverworldSubmapColorPalette {
            global_palette: Rc::clone(gp),
            layer2:         match ow_state {
                OverworldState::PreSpecial => &self.layer2_pre_special,
                OverworldState::PostSpecial => &self.layer2_post_special,
            }
            .get(i_submap_palette)
            .cloned()
            .ok_or(OverworldSubmapPaletteError::Layer2)?,
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

impl_color_palette!(GlobalOverworldColorPalette {
    [0x2..=0x7, 0x9..=0xF] => layer1,
    [0x0..=0x1, 0x8..=0xF] => layer3,
    [0x9..=0xF, 0x1..=0x7] => sprite,
});

impl_color_palette!(OverworldSubmapColorPalette {
    [0x4..=0x7, 0x1..=0x7] => layer2,
    _ => global_palette,
});
