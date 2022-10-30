use std::{borrow::Borrow, cell::RefCell};

use eframe::egui::{
    panel::Side,
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
use smwe_rom::graphics::palette::ColorPalette;
use smwe_widgets::flipbook::{AnimationState, Flipbook};

use crate::{frame_context::FrameContext, ui::tool::UiTool};

pub struct UiTiles16x16 {
    tile_images:      Vec<TextureHandle>,
    selected_tileset: u32,
    level_number:     u32,

    test_anim: Option<AnimationState>,
}

impl Default for UiTiles16x16 {
    fn default() -> Self {
        Self { tile_images: Vec::new(), selected_tileset: 1, level_number: 0, test_anim: None }
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
                                let res = ui.selectable_value(&mut self.selected_tileset, i, format!("Tileset {i}"));
                                if res.clicked() {
                                    self.load_images(ctx);
                                }
                            }
                        });

                    ui.horizontal(|ui| {
                        let res = ui.add({
                            DragValue::new(&mut self.level_number).clamp_range(0..=0x1FF).hexadecimal(3, false, true)
                        });
                        ui.label("Level number");

                        if res.changed() {
                            self.load_images(ctx);
                        }
                    });

                    // TODO: delete, this is a proof of concept
                    ui.add(Flipbook::new(self.test_anim.as_mut().unwrap(), [32.0, 32.0]).fps(15.0).looped(true));
                });

                let block_size = tweak!(32.0);
                ScrollArea::vertical().min_scrolled_height(ui.available_height()).show(ui, |ui| {
                    TableBuilder::new(ui).columns(Size::exact(block_size + tweak!(5.0)), 16).body(|tb| {
                        tb.rows(block_size + tweak!(17.0), self.tile_images.len() / 16, |row, mut tr| {
                            for tile in (row * 16)..((row * 16) + 16).min(0x200) {
                                tr.col(|ui| {
                                    ui.label(format!("{tile:X}"));
                                    ui.image(&self.tile_images[tile], Vec2::splat(block_size));
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
            let map16_gfx = map16_tile
                .gfx(&rom.map16_tilesets.gfx_list, &rom.gfx_files, self.selected_tileset as usize - 1)
                .into_iter()
                .zip(tiles_8x8)
                .map(|(gfx, m16)| {
                    let palette_row = palette.get_row(m16.palette() as usize);
                    gfx.to_rgba(&palette_row)
                })
                .collect_vec();
            assert_eq!(map16_gfx.len(), 4);

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

            self.tile_images.push(ctx.egui_ctx.load_texture(
                format!("map16 {map16_id}"),
                image,
                TextureFilter::Nearest,
            ));
        }

        self.test_anim =
            Some(AnimationState::from_texture_handles(self.tile_images.clone(), Id::from("test anim")).unwrap());
    }
}
