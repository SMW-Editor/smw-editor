use std::{convert::TryInto, ops::RangeInclusive};

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
    error::{ColorPaletteError, RomParseError},
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
        $(, default_color_1 => $default_color_1:expr)? $(,)?
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
            }

            #[allow(unreachable_code)]
            fn get_color_at(&self, row: usize, col: usize) -> Option<Abgr1555> {
                if row > 0xF || col > 0xF {
                    None
                } else {
                    $(
                        if $rows.contains(&row) && $cols.contains(&col) {
                            let ri = row - *$rows.start();
                            let ci = col - *$cols.start();
                            return Some(self.$field_name[(ri * $cols.count()) + ci]);
                        }
                    )+
                    $(if col == 0 {
                        Some(Abgr1555::TRANSPARENT)
                    } else if col == 1 {
                        Some($default_color_1)
                    } else)? {
                        Some(Abgr1555::TRANSPARENT)
                    }
                }
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct ColorPalettes {
    pub players: Box<[Abgr1555]>,

    pub ow_layer1: Box<[Abgr1555]>,
    pub ow_layer3: Box<[Abgr1555]>,
    pub ow_sprite: Box<[Abgr1555]>,

    pub wtf:         Box<[Abgr1555]>,
    pub lv_layer3:   Box<[Abgr1555]>,
    pub lv_berry:    Box<[Abgr1555]>,
    pub lv_animated: Box<[Abgr1555]>,

    pub ow_specific_set: OverworldColorPaletteSet,
    pub lv_specific_set: LevelColorPaletteSet,
}

#[derive(Clone)]
pub struct OverworldColorPaletteSet {
    pub layer2_pre_special:  Vec<Box<[Abgr1555]>>,
    pub layer2_post_special: Vec<Box<[Abgr1555]>>,
    pub layer2_indices:      Vec<usize>,
}

#[derive(Clone)]
pub struct LevelColorPaletteSet {
    pub back_area_colors: Vec<Abgr1555>,
    pub bg_palettes:      Vec<Box<[Abgr1555]>>,
    pub fg_palettes:      Vec<Box<[Abgr1555]>>,
    pub sprite_palettes:  Vec<Box<[Abgr1555]>>,
}

#[derive(Clone)]
pub struct SpecificLevelColorPalette {
    pub back_area_color: Abgr1555,
    pub background:      Box<[Abgr1555]>,
    pub foreground:      Box<[Abgr1555]>,
    pub sprite:          Box<[Abgr1555]>,
    pub players:         Box<[Abgr1555]>,
    pub wtf:             Box<[Abgr1555]>,
    pub layer3:          Box<[Abgr1555]>,
    pub berry:           Box<[Abgr1555]>,
    pub animated:        Box<[Abgr1555]>,
}

#[derive(Clone)]
pub struct SpecificOverworldColorPalette {
    pub layer1:  Box<[Abgr1555]>,
    pub layer2:  Box<[Abgr1555]>,
    pub layer3:  Box<[Abgr1555]>,
    pub sprite:  Box<[Abgr1555]>,
    pub players: Box<[Abgr1555]>,
    pub wtf:     Box<[Abgr1555]>,
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

impl ColorPalettes {
    pub fn parse(rom_data: &[u8], levels: &[Level]) -> Result<Self, RomParseError> {
        match Self::parse_impl(rom_data, levels) {
            Ok((_, palette_set)) => Ok(palette_set),
            Err(_) => Err(RomParseError::ColorPalettes),
        }
    }

    fn parse_impl<'r>(rom_data: &'r [u8], levels: &[Level]) -> IResult<&'r [u8], Self> {
        const PLAYER_PALETTE: AddrSnes = AddrSnes(0x00B2C8);
        const OW_LAYER1_PALETTES: AddrSnes = AddrSnes(0x00B528);
        const OW_LAYER3_PALETTES: AddrSnes = AddrSnes(0x00B5EC);
        const OW_SPRITE_PALETTES: AddrSnes = AddrSnes(0x00B58A);
        const LV_WTF_PALETTE: AddrSnes = AddrSnes(0x00B250);
        const LV_LAYER3_PALETTE: AddrSnes = AddrSnes(0x00B170);
        const LV_BERRY_PALETTE: AddrSnes = AddrSnes(0x00B674);
        const LV_ANIMATED_COLOR: AddrSnes = AddrSnes(0x00B60C);

        let parse_colors = make_color_parser(rom_data);

        let (_, players) = parse_colors(PLAYER_PALETTE, PALETTE_PLAYER_LENGTH)?;
        let (_, ow_layer1_colors) = parse_colors(OW_LAYER1_PALETTES, OW_LAYER1_PALETTE_LENGTH)?;
        let (_, ow_layer3_colors) = parse_colors(OW_LAYER3_PALETTES, OW_LAYER3_PALETTE_LENGTH)?;
        let (_, ow_sprite_colors) = parse_colors(OW_SPRITE_PALETTES, OW_SPRITE_PALETTE_LENGTH)?;
        let (_, lv_wtf) = parse_colors(LV_WTF_PALETTE, PALETTE_WTF_LENGTH)?;
        let (_, lv_layer3) = parse_colors(LV_LAYER3_PALETTE, PALETTE_LAYER3_LENGTH)?;
        let (_, lv_berry) = parse_colors(LV_BERRY_PALETTE, PALETTE_BERRY_LENGTH)?;
        let (_, lv_animated) = parse_colors(LV_ANIMATED_COLOR, PALETTE_ANIMATED_LENGTH)?;
        let (_, lv_specific_set) = LevelColorPaletteSet::parse(rom_data, levels)?;
        let (_, ow_specific_set) = OverworldColorPaletteSet::parse(rom_data)?;

        Ok((rom_data, ColorPalettes {
            players: players.into(),
            ow_layer1: ow_layer1_colors.into(),
            ow_layer3: ow_layer3_colors.into(),
            ow_sprite: ow_sprite_colors.into(),
            wtf: lv_wtf.into(),
            lv_layer3: lv_layer3.into(),
            lv_berry: lv_berry.into(),
            lv_animated: lv_animated.into(),
            ow_specific_set,
            lv_specific_set,
        }))
    }

    pub fn get_level_palette(&self, header: &PrimaryHeader) -> Result<SpecificLevelColorPalette, ColorPaletteError> {
        self.lv_specific_set.get_level_palette(header, &self)
    }

    pub fn get_submap_palette(
        &self, submap: usize, ow_state: OverworldState,
    ) -> Result<SpecificOverworldColorPalette, ColorPaletteError> {
        self.ow_specific_set.get_submap_palette(submap, ow_state, &self)
    }
}

impl LevelColorPaletteSet {
    fn parse<'a>(rom_data: &'a [u8], levels: &[Level]) -> IResult<&'a [u8], LevelColorPaletteSet> {
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
        }

        Ok((rom_data, palette_set))
    }

    pub fn get_level_palette(
        &self, header: &PrimaryHeader, palettes: &ColorPalettes,
    ) -> Result<SpecificLevelColorPalette, ColorPaletteError> {
        let i_back_area_color = header.back_area_color as usize;
        let i_background = header.palette_bg as usize;
        let i_foreground = header.palette_fg as usize;
        let i_sprite = header.palette_sprite as usize;
        self.palette_from_indices(i_back_area_color, i_background, i_foreground, i_sprite, palettes)
    }

    pub fn palette_from_indices(
        &self, i_back_area_color: usize, i_background: usize, i_foreground: usize, i_sprite: usize,
        palettes: &ColorPalettes,
    ) -> Result<SpecificLevelColorPalette, ColorPaletteError> {
        Ok(SpecificLevelColorPalette {
            back_area_color: self
                .back_area_colors
                .get(i_back_area_color)
                .cloned()
                .ok_or(ColorPaletteError::LvBackAreaColor)?,
            background:      self.bg_palettes.get(i_background).cloned().ok_or(ColorPaletteError::LvBackground)?,
            foreground:      self.fg_palettes.get(i_foreground).cloned().ok_or(ColorPaletteError::LvForeground)?,
            sprite:          self.sprite_palettes.get(i_sprite).cloned().ok_or(ColorPaletteError::LvSprite)?,
            wtf:             palettes.wtf.clone(),
            layer3:          palettes.lv_layer3.clone(),
            berry:           palettes.lv_berry.clone(),
            animated:        palettes.lv_animated.clone(),
            players:         palettes.players.clone(),
        })
    }
}

impl OverworldColorPaletteSet {
    fn parse(rom_data: &[u8]) -> IResult<&[u8], OverworldColorPaletteSet> {
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
        &self, submap: usize, ow_state: OverworldState, palettes: &ColorPalettes,
    ) -> Result<SpecificOverworldColorPalette, ColorPaletteError> {
        let i_submap_palette = *self.layer2_indices.get(submap).ok_or(ColorPaletteError::OwLayer2)?;
        self.get_submap_palette_from_indices(i_submap_palette, ow_state, palettes)
    }

    pub fn get_submap_palette_from_indices(
        &self, i_submap_palette: usize, ow_state: OverworldState, palettes: &ColorPalettes,
    ) -> Result<SpecificOverworldColorPalette, ColorPaletteError> {
        let mut palette = SpecificOverworldColorPalette {
            layer1:  palettes.ow_layer1.clone(),
            layer2:  match ow_state {
                OverworldState::PreSpecial => &self.layer2_pre_special,
                OverworldState::PostSpecial => &self.layer2_post_special,
            }
            .get(i_submap_palette)
            .cloned()
            .ok_or(ColorPaletteError::OwLayer2)?,
            layer3:  palettes.ow_layer3.clone(),
            sprite:  palettes.ow_sprite.clone(),
            players: palettes.players.clone(),
            wtf:     palettes.wtf.clone()[23..=27].into(),
        };
        palette.wtf[0] = Abgr1555::WHITE;
        Ok(palette)
    }
}

impl_color_palette!(SpecificLevelColorPalette {
    [0x0..=0x1, 0x2..=0x7] => background,
    [0x2..=0x3, 0x2..=0x7] => foreground,
    [0xE..=0xF, 0x2..=0x7] => sprite,
    [0x4..=0xD, 0x2..=0x7] => wtf,
    [0x0..=0x1, 0x8..=0xF] => layer3,
    [0x2..=0x4, 0x9..=0xF] => berry,
    [0x9..=0xB, 0x9..=0xF] => berry,
    [0x8..=0x8, 0x6..=0xF] => players,
    default_color_1 => Abgr1555::WHITE,
});

impl_color_palette!(SpecificOverworldColorPalette {
    [0x2..=0x7, 0x9..=0xF] => layer1,
    [0x4..=0x7, 0x1..=0x7] => layer2,
    [0x0..=0x1, 0x8..=0xF] => layer3,
    [0x9..=0xF, 0x1..=0x7] => sprite,
    [0x8..=0x8, 0x6..=0xF] => players,
    [0x8..=0x8, 0x1..=0x5] => wtf,
});
