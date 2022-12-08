use constants::*;
use eframe::egui::{Color32, ColorImage, DragValue, ScrollArea, SidePanel, TextureHandle, TextureOptions, Ui, Window};
use smwe_rom::graphics::{gfx_file::TileFormat, palette::ColorPalette};

use crate::{frame_context::FrameContext, ui::tool::UiTool};

#[allow(dead_code)]
#[rustfmt::skip]
mod constants {
    pub const N_TILES_IN_ROW: usize = 16;

    pub const I_PAL_LEVEL_WTF:        usize = 0;
    pub const I_PAL_LEVEL_PLAYERS:    usize = 1;
    pub const I_PAL_LEVEL_LAYER3:     usize = 2;
    pub const I_PAL_LEVEL_BERRY:      usize = 3;
    pub const I_PAL_LEVEL_BACKGROUND: usize = 4;
    pub const I_PAL_LEVEL_FOREGROUND: usize = 5;
    pub const I_PAL_LEVEL_SPRITE:     usize = 6;
    pub const PALETTE_CATEGORIES: [&str; 7] = [
        "Level: Wtf",
        "Level: Players",
        "Level: Layer3",
        "Level: Berry",
        "Level: Background",
        "Level: Foreground",
        "Level: Sprite",
    ];
}

pub struct UiGfxViewer {
    image_handle:         Option<TextureHandle>,
    curr_image_size:      (usize, usize),
    curr_image_format:    TileFormat,
    curr_gfx_file_num:    i32,
    curr_palette_row_idx: i32,
    curr_bg_palette_num:  i32,
    curr_fg_palette_num:  i32,
    curr_sp_palette_num:  i32,
    curr_pl_palette_num:  i32,
}

impl Default for UiGfxViewer {
    fn default() -> Self {
        log::info!("Opened GFX Viewer");
        UiGfxViewer {
            image_handle:         None,
            curr_image_size:      (0, 0),
            curr_image_format:    TileFormat::Tile2bpp,
            curr_gfx_file_num:    0,
            curr_palette_row_idx: 0,
            curr_bg_palette_num:  0,
            curr_fg_palette_num:  0,
            curr_sp_palette_num:  0,
            curr_pl_palette_num:  0,
        }
    }
}

impl UiTool for UiGfxViewer {
    fn update(&mut self, ui: &mut Ui, ctx: &mut FrameContext) -> bool {
        if self.image_handle.is_none() {
            self.update_image(ctx);
        }

        let mut running = true;
        Window::new("GFX Viewer") //
            .default_height(600.0)
            .open(&mut running)
            .show(ui.ctx(), |ui| {
                SidePanel::left("switches_panel").show_inside(ui, |ui| self.switches(ui, ctx));
                self.gfx_image(ui);
            });

        if !running {
            log::info!("Closed GFX Viewer");
        }
        running
    }
}

impl UiGfxViewer {
    fn switches(&mut self, ui: &mut Ui, ctx: &mut FrameContext) {
        let mut changed_any = false;
        if let Some(project) = ctx.project_ref.as_ref() {
            let project = project.borrow();
            let rom = &project.rom_data;

            let level_count = rom.gfx_files.len() as i32;
            let palette_row_count = 16i32;
            let bg_pals_count = rom.color_palettes.lv_specific_set.bg_palettes.len() as i32;
            let fg_pals_count = rom.color_palettes.lv_specific_set.fg_palettes.len() as i32;
            let sp_pals_count = rom.color_palettes.lv_specific_set.sprite_palettes.len() as i32;
            let pl_pals_count = rom.color_palettes.players.len() as i32 / 10;

            let inputs = [
                ("GFX file number", &mut self.curr_gfx_file_num, level_count),
                ("Palette row index", &mut self.curr_palette_row_idx, palette_row_count),
                ("Background palette index", &mut self.curr_bg_palette_num, bg_pals_count),
                ("Foreground palette index", &mut self.curr_fg_palette_num, fg_pals_count),
                ("Sprite palette index", &mut self.curr_sp_palette_num, sp_pals_count),
                ("Player palette index", &mut self.curr_pl_palette_num, pl_pals_count),
            ];

            ui.vertical(|ui| {
                for (label, var, max) in inputs {
                    ui.horizontal(|ui| {
                        let dv = DragValue::new(var).hexadecimal(2, false, true).clamp_range(0..=max - 1);
                        changed_any |= ui.add(dv).changed();
                        ui.label(label);
                    });
                }
                ui.label(format!("{} x {} px", self.curr_image_size.0, self.curr_image_size.1));
                ui.label(match self.curr_image_format {
                    TileFormat::Tile2bpp => "2BPP",
                    TileFormat::Tile3bpp => "3BPP",
                    TileFormat::Tile4bpp => "4BPP",
                    TileFormat::Tile8bpp => "8BPP",
                    TileFormat::Tile3bppMode7 => "Mode7",
                });
            });
        }

        if changed_any {
            self.update_image(ctx);
        }
    }

    fn gfx_image(&mut self, ui: &mut Ui) {
        if let Some(handle) = &self.image_handle {
            let (img_w, img_h) = self.curr_image_size;
            ScrollArea::vertical().min_scrolled_height(ui.available_height()).show(ui, |ui| {
                ui.image(handle, [(img_w * 3) as f32, (img_h * 3) as f32]);
            });
        }
    }

    fn update_image(&mut self, ctx: &mut FrameContext) {
        let project = ctx.project_ref.as_ref().unwrap().borrow();
        let rom = &project.rom_data;
        let gfx_file = &rom.gfx_files[self.curr_gfx_file_num as usize];

        let img_w = (gfx_file.tiles.len() * 8).clamp(8, N_TILES_IN_ROW * 8);
        let img_h = ((1 + (gfx_file.tiles.len() / N_TILES_IN_ROW)) * 8).max(8);
        self.curr_image_size = (img_w, img_h);
        self.curr_image_format = gfx_file.tile_format;

        let palette = &rom
            .color_palettes
            .lv_specific_set
            .palette_from_indices(
                0,
                self.curr_bg_palette_num as usize,
                self.curr_fg_palette_num as usize,
                self.curr_sp_palette_num as usize,
                &rom.color_palettes,
            )
            .unwrap();

        let mut new_image = ColorImage::new([img_w, img_h], Color32::TRANSPARENT);
        let palette = palette.get_row(self.curr_palette_row_idx as usize);
        for (tile_idx, tile) in gfx_file.tiles.iter().enumerate() {
            let (row, column) = (tile_idx / N_TILES_IN_ROW, tile_idx % N_TILES_IN_ROW);
            let (tile_x, tile_y) = (column * 8, row * 8);
            let rgba_tile = tile.to_rgba(&palette);
            for (pixel_idx, &color) in rgba_tile.iter().enumerate() {
                let (pixel_x, pixel_y) = (tile_x + (pixel_idx % 8), tile_y + (pixel_idx / 8));
                new_image[(pixel_x, pixel_y)] = Color32::from(color);
            }
        }
        self.image_handle = Some(ctx.egui_ctx.load_texture("gfx-file-image", new_image, TextureOptions::NEAREST));

        log::info!("Successfully created a GFX file image (w = {img_w}, h = {img_h}).");
    }
}
