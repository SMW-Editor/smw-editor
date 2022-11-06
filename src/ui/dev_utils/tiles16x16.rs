use std::{borrow::Borrow, cell::RefCell};

use eframe::egui::{
    panel::Side,
    vec2,
    Color32,
    ColorImage,
    ComboBox,
    DragValue,
    Id,
    Rgba,
    ScrollArea,
    SidePanel,
    TextureFilter,
    TextureHandle,
    Ui,
    Vec2,
    Window,
};
use egui_extras::{Size, TableBuilder};
use inline_tweak::tweak;
use itertools::Itertools;
use smwe_project::Project;
use smwe_rom::{graphics::palette::ColorPalette, objects::map16::Tile8x8};
use smwe_widgets::flipbook::{AnimationState, Flipbook};

use crate::{frame_context::FrameContext, ui::tool::UiTool};

enum TileImage {
    Static(TextureHandle),
    Animated(AnimationState),
}

pub struct UiTiles16x16 {
    tile_images:      Vec<TileImage>,
    selected_tileset: u32,
    level_number:     u32,
}

impl Default for UiTiles16x16 {
    fn default() -> Self {
        Self { tile_images: Vec::new(), selected_tileset: 1, level_number: 0 }
    }
}

impl UiTool for UiTiles16x16 {
    fn update(&mut self, ui: &mut Ui, ctx: &mut FrameContext) -> bool {
        if self.tile_images.is_empty() {
            self.load_images(ctx);
        }

        let mut running = true;

        Window::new("16x16 Tiles") //
            .default_height(600.0)
            .open(&mut running)
            .show(ui.ctx(), |ui| {
                SidePanel::new(Side::Left, ui.id().with("sp")).show_inside(ui, |ui| {
                    ComboBox::new(ui.id().with("cb Tileset"), "Tileset")
                        .selected_text(format!("Tileset {}", self.selected_tileset))
                        .show_ui(ui, |ui| {
                            for i in 1..=5 {
                                let response =
                                    ui.selectable_value(&mut self.selected_tileset, i, format!("Tileset {i}"));
                                if response.clicked() {
                                    self.load_images(ctx);
                                }
                            }
                        });

                    ui.horizontal(|ui| {
                        let response = ui.add({
                            DragValue::new(&mut self.level_number).clamp_range(0..=0x1FF).hexadecimal(3, false, true)
                        });
                        ui.label("Level number");

                        if response.changed() {
                            self.load_images(ctx);
                        }
                    });
                });

                let block_size = Vec2::splat(tweak!(32.0));
                let cell_padding = vec2(tweak!(5.0), tweak!(19.0));
                ScrollArea::vertical().min_scrolled_height(ui.available_height()).show(ui, |ui| {
                    TableBuilder::new(ui).columns(Size::exact(block_size.x + cell_padding.x), 16).body(|tb| {
                        tb.rows(block_size.y + cell_padding.y, self.tile_images.len() / 16, |row, mut tr| {
                            for tile in (row * 16)..((row * 16) + 16).min(0x200) {
                                tr.col(|ui| {
                                    ui.label(format!("{tile:X}"));
                                    match &mut self.tile_images[tile] {
                                        TileImage::Static(texture_id) => {
                                            ui.image(texture_id, block_size);
                                        }
                                        TileImage::Animated(animation) => {
                                            ui.add(Flipbook::new(animation, block_size).looped(true).fps(tweak!(16.0)));
                                        }
                                    }
                                });
                            }
                        });
                    });
                });
            });

        if !running {
            log::info!("Closed Tiles16x16");
        }
        running
    }
}

impl UiTiles16x16 {
    fn load_images(&mut self, ctx: &mut FrameContext) {
        self.tile_images.clear();

        let project: &RefCell<Project> = ctx.project_ref.as_ref().unwrap();
        let rom = &project.borrow().rom_data;

        let level = &rom.levels[self.level_number as usize];
        let palette = rom.color_palettes.get_level_palette(&level.primary_header).unwrap();

        for map16_id in 0..=0x1FF {
            let map16_tile = rom.map16_tilesets.get_map16_tile(map16_id, self.selected_tileset as usize - 1).unwrap();
            let tiles_8x8 =
                [map16_tile.upper_left, map16_tile.lower_left, map16_tile.upper_right, map16_tile.lower_right];

            let make_tile_iter = || {
                map16_tile
                    .gfx(&rom.map16_tilesets.gfx_list, &rom.gfx_files, self.selected_tileset as usize - 1)
                    .into_iter()
                    .zip(tiles_8x8)
            };

            let has_animated_color =
                make_tile_iter().any(|(gfx, m16)| m16.palette() == 6 && gfx.color_indices.iter().any(|&idx| idx == 4));

            if has_animated_color {
                let mut frames = Vec::with_capacity(palette.animated.len());
                for &animated_color in palette.animated.iter() {
                    let map16_gfx = make_tile_iter()
                        .map(|(gfx, m16)| {
                            let palette_row = palette.get_row(m16.palette() as usize);
                            if m16.palette() == 6 {
                                gfx.to_rgba_with_substitute_at(&palette_row, animated_color, 4)
                            } else {
                                gfx.to_rgba(&palette_row)
                            }
                        })
                        .collect_vec();
                    assert_eq!(map16_gfx.len(), 4);
                    let frame_image = Self::make_image(&tiles_8x8, &map16_gfx);
                    frames.push(frame_image);
                }
                let animation = AnimationState::from_frames(frames, Id::new(format!("map16_{map16_id}")), ctx.egui_ctx)
                    .unwrap_or_else(|e| panic!("Cannot assemble animation for tile {map16_id}: {e}"));
                self.tile_images.push(TileImage::Animated(animation));
            } else {
                let map16_gfx = make_tile_iter()
                    .map(|(gfx, m16)| {
                        let palette_row = palette.get_row(m16.palette() as usize);
                        gfx.to_rgba(&palette_row)
                    })
                    .collect_vec();
                assert_eq!(map16_gfx.len(), 4);
                let image = ctx.egui_ctx.load_texture(
                    format!("map16 {map16_id}"),
                    Self::make_image(&tiles_8x8, &map16_gfx),
                    TextureFilter::Nearest,
                );
                self.tile_images.push(TileImage::Static(image));
            }
        }
    }

    fn make_image(tiles_8x8: &[Tile8x8; 4], map16_gfx: &[Box<[Rgba]>]) -> ColorImage {
        let mut image = ColorImage::new([16, 16], Color32::TRANSPARENT);
        for y in 0..image.size[1] {
            for x in 0..image.size[0] {
                let ti = (y / 8) + (2 * (x / 8));
                let tile_8x8 = tiles_8x8[ti];
                let tile_gfx: &[Rgba] = map16_gfx[ti].borrow();
                let [ty, tx] = [
                    if tile_8x8.flip_y() { 7 - (y % 8) } else { y % 8 },
                    if tile_8x8.flip_x() { 7 - (x % 8) } else { x % 8 },
                ];
                image[(x, y)] = Color32::from(tile_gfx[(8 * ty) + tx]);
            }
        }
        image
    }
}
