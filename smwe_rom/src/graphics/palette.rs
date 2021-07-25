use std::{convert::TryInto, ops::RangeInclusive};

use nom::{combinator::map, multi::many0, number::complete::le_u16};

use crate::{
    error::{ColorPaletteError, ColorPaletteParseError},
    graphics::color::{Abgr1555, ABGR1555_SIZE},
    level::{headers::PrimaryHeader, Level},
    snes_utils::{addr::AddrSnes, rom::Rom, rom_slice::SnesSlice},
};

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

fn make_color_parser(
    rom: &Rom,
) -> impl Fn(SnesSlice, ColorPaletteParseError) -> Result<Vec<Abgr1555>, ColorPaletteParseError> + '_ {
    move |slice, err| {
        let read_colors = many0(map(le_u16, Abgr1555));
        rom.parse_slice_lorom(slice, read_colors).map_err(|_| err)
    }
}

impl ColorPalettes {
    pub fn parse(rom: &Rom, levels: &[Level]) -> Result<Self, ColorPaletteParseError> {
        const PLAYER_PALETTE: SnesSlice = SnesSlice::new(AddrSnes(0x00B2C8), 4 * 0x14);
        const OW_LAYER1_PALETTES: SnesSlice = SnesSlice::new(AddrSnes(0x00B528), ABGR1555_SIZE * 7 * 6);
        const OW_LAYER3_PALETTES: SnesSlice = SnesSlice::new(AddrSnes(0x00B5EC), ABGR1555_SIZE * 8 * 2);
        const OW_SPRITE_PALETTES: SnesSlice = SnesSlice::new(AddrSnes(0x00B58A), ABGR1555_SIZE * 7 * 7);
        const LV_WTF_PALETTE: SnesSlice =
            SnesSlice::new(AddrSnes(0x00B250), ABGR1555_SIZE * ((0xD - 0x4 + 1) * (0x7 - 0x2 + 1)));
        const LV_LAYER3_PALETTE: SnesSlice = SnesSlice::new(AddrSnes(0x00B170), 0x20);
        const LV_BERRY_PALETTE: SnesSlice = SnesSlice::new(AddrSnes(0x00B674), 3 * 0x0E);
        const LV_ANIMATED_COLOR: SnesSlice = SnesSlice::new(AddrSnes(0x00B60C), ABGR1555_SIZE * 8);

        let parse_colors = make_color_parser(rom);

        let players = parse_colors(PLAYER_PALETTE, ColorPaletteParseError::PlayerPalette)?;
        let ow_layer1_colors = parse_colors(OW_LAYER1_PALETTES, ColorPaletteParseError::OverworldLayer1Palette)?;
        let ow_layer3_colors = parse_colors(OW_LAYER3_PALETTES, ColorPaletteParseError::OverworldLayer3Palette)?;
        let ow_sprite_colors = parse_colors(OW_SPRITE_PALETTES, ColorPaletteParseError::OverworldSpritePalette)?;
        let lv_wtf = parse_colors(LV_WTF_PALETTE, ColorPaletteParseError::LevelMiscPalette)?;
        let lv_layer3 = parse_colors(LV_LAYER3_PALETTE, ColorPaletteParseError::LevelLayer3Palette)?;
        let lv_berry = parse_colors(LV_BERRY_PALETTE, ColorPaletteParseError::LevelBerryPalette)?;
        let lv_animated = parse_colors(LV_ANIMATED_COLOR, ColorPaletteParseError::LevelAnimatedColor)?;
        let lv_specific_set = LevelColorPaletteSet::parse(rom, levels)?;
        let ow_specific_set = OverworldColorPaletteSet::parse(rom)?;

        Ok(ColorPalettes {
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
        })
    }

    pub fn get_level_palette(&self, header: &PrimaryHeader) -> Result<SpecificLevelColorPalette, ColorPaletteError> {
        self.lv_specific_set.get_level_palette(header, self)
    }

    pub fn get_submap_palette(
        &self, submap: usize, ow_state: OverworldState,
    ) -> Result<SpecificOverworldColorPalette, ColorPaletteError> {
        self.ow_specific_set.get_submap_palette(submap, ow_state, self)
    }
}

impl LevelColorPaletteSet {
    fn parse(rom: &Rom, levels: &[Level]) -> Result<Self, ColorPaletteParseError> {
        const BACK_AREA_COLORS: SnesSlice = SnesSlice::new(AddrSnes(0x00B0A0), ABGR1555_SIZE);
        const BG_PALETTES: SnesSlice = SnesSlice::new(AddrSnes(0x00B0B0), 0x18);
        const FG_PALETTES: SnesSlice = SnesSlice::new(AddrSnes(0x00B190), 0x18);
        const SPRITE_PALETTES: SnesSlice = SnesSlice::new(AddrSnes(0x00B318), 0x18);

        let parse_colors = make_color_parser(rom);

        let mut palette_set = Self {
            back_area_colors: Vec::with_capacity(8),
            bg_palettes:      Vec::with_capacity(8),
            fg_palettes:      Vec::with_capacity(8),
            sprite_palettes:  Vec::with_capacity(8),
        };

        for (level_num, level) in levels.iter().enumerate() {
            let header = &level.primary_header;

            let idx_bc = header.back_area_color() as usize;
            let idx_bg = header.palette_bg() as usize;
            let idx_fg = header.palette_fg() as usize;
            let idx_sp = header.palette_sprite() as usize;

            let bc = parse_colors(
                BACK_AREA_COLORS.skip_forward(idx_bc),
                ColorPaletteParseError::LevelBackAreaColor(level_num),
            )?;
            let bg = parse_colors(
                BG_PALETTES.skip_forward(idx_bg),
                ColorPaletteParseError::LevelBackgroundPalette(level_num),
            )?;
            let fg = parse_colors(
                FG_PALETTES.skip_forward(idx_fg),
                ColorPaletteParseError::LevelForegroundPalette(level_num),
            )?;
            let sp = parse_colors(
                SPRITE_PALETTES.skip_forward(idx_sp),
                ColorPaletteParseError::LevelSpritePalette(level_num),
            )?;

            if palette_set.back_area_colors.len() < idx_bc + 1 {
                let value = Abgr1555::default();
                palette_set.back_area_colors.resize(idx_bc + 1, value);
            }
            if palette_set.bg_palettes.len() < idx_bg + 1 {
                let value: Box<[Abgr1555]> = [Abgr1555::default(); BG_PALETTES.size / ABGR1555_SIZE].into();
                palette_set.bg_palettes.resize(idx_bg + 1, value);
            }
            if palette_set.fg_palettes.len() < idx_fg + 1 {
                let value: Box<[Abgr1555]> = [Abgr1555::default(); FG_PALETTES.size / ABGR1555_SIZE].into();
                palette_set.fg_palettes.resize(idx_fg + 1, value);
            }
            if palette_set.sprite_palettes.len() < idx_sp + 1 {
                let value: Box<[Abgr1555]> = [Abgr1555::default(); SPRITE_PALETTES.size / ABGR1555_SIZE].into();
                palette_set.sprite_palettes.resize(idx_sp + 1, value);
            }

            palette_set.back_area_colors[idx_bc] = bc[0];
            palette_set.bg_palettes[idx_bg] = bg.try_into().unwrap();
            palette_set.fg_palettes[idx_fg] = fg.try_into().unwrap();
            palette_set.sprite_palettes[idx_sp] = sp.try_into().unwrap();
        }

        Ok(palette_set)
    }

    pub fn get_level_palette(
        &self, header: &PrimaryHeader, palettes: &ColorPalettes,
    ) -> Result<SpecificLevelColorPalette, ColorPaletteError> {
        let i_back_area_color = header.back_area_color() as usize;
        let i_background = header.palette_bg() as usize;
        let i_foreground = header.palette_fg() as usize;
        let i_sprite = header.palette_sprite() as usize;
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
    fn parse(rom: &Rom) -> Result<OverworldColorPaletteSet, ColorPaletteParseError> {
        let parse_colors = make_color_parser(rom);

        const LAYER2_NORMAL_PALETTES: SnesSlice = SnesSlice::new(AddrSnes(0x00B3D8), ABGR1555_SIZE * 7 * 4);
        const LAYER2_SPECIAL_PALETTES: SnesSlice = SnesSlice::new(AddrSnes(0x00B732), ABGR1555_SIZE * 7 * 4);
        const LAYER2_PALETTE_INDIRECT1: SnesSlice = SnesSlice::new(AddrSnes(0x00AD1E), 7);
        const LAYER2_PALETTE_INDIRECT2: SnesSlice = SnesSlice::new(AddrSnes(0x00ABDF), 7);

        let mut layer2_pre_special = Vec::with_capacity(6);
        let mut layer2_post_special = Vec::with_capacity(6);
        let mut layer2_indices = Vec::with_capacity(7);

        for i in 0..6 {
            let subworld_pal_idx = i * 14 * 4;
            let layer2_colors_normal = parse_colors(
                LAYER2_NORMAL_PALETTES.offset_forward(subworld_pal_idx),
                ColorPaletteParseError::OverworldLayer2NormalPalette(i),
            )?;
            let layer2_colors_special = parse_colors(
                LAYER2_SPECIAL_PALETTES.offset_forward(subworld_pal_idx),
                ColorPaletteParseError::OverworldLayer2SpecialPalette(i),
            )?;

            layer2_pre_special.push(layer2_colors_normal.into());
            layer2_post_special.push(layer2_colors_special.into());
        }

        let indirect_table_1 = rom
            .slice_lorom(LAYER2_PALETTE_INDIRECT1)
            .map_err(|_| ColorPaletteParseError::OverworldLayer2IndicesIndirect1Read(LAYER2_PALETTE_INDIRECT1))?;

        for &offset in indirect_table_1 {
            let index_offset = LAYER2_PALETTE_INDIRECT2.offset_forward(2 * offset as usize).begin;
            let ptr16_slice = SnesSlice::new(index_offset, 2);
            let ptr16 = rom
                .parse_slice_lorom(ptr16_slice, le_u16)
                .map_err(|_| ColorPaletteParseError::OverworldLayer2IndexRead(offset as usize))?;

            let idx = ptr16 / 0x38;
            layer2_indices.push(idx as usize);
        }

        Ok(Self { layer2_pre_special, layer2_post_special, layer2_indices })
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
        let layer2_pal = match ow_state {
            OverworldState::PreSpecial => &self.layer2_pre_special,
            OverworldState::PostSpecial => &self.layer2_post_special,
        };
        let mut palette = SpecificOverworldColorPalette {
            layer1:  palettes.ow_layer1.clone(),
            layer2:  layer2_pal.get(i_submap_palette).cloned().ok_or(ColorPaletteError::OwLayer2)?,
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
