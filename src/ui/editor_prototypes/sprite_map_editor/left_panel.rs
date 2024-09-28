use std::sync::Arc;

use egui::*;
use egui_glow::CallbackFn;
use inline_tweak::tweak;
use smwe_math::coordinates::OnCanvas;
use smwe_render::tile_renderer::TileUniforms;
use smwe_widgets::{
    palette_view::{PaletteView, SelectionType, ViewedPalettes},
    vram_view::{ViewedVramTiles, VramSelectionMode, VramView},
};

use super::UiSpriteMapEditor;

impl UiSpriteMapEditor {
    pub(super) fn left_panel(&mut self, ui: &mut Ui) {
        ScrollArea::vertical().min_scrolled_height(ui.available_height()).show(ui, |ui| {
            ui.add_space(ui.spacing().item_spacing.y);
            ui.group(|ui| {
                ui.allocate_space(vec2(ui.available_width(), 0.));
                self.tile_selector(ui);
                ui.add_space(ui.spacing().item_spacing.y);
                self.tile_selection_preview(ui);
            });

            ui.add_space(ui.spacing().item_spacing.y);
            ui.group(|ui| {
                ui.allocate_space(vec2(ui.available_width(), 0.));
                self.palette_row_selector(ui);
            });

            if cfg!(debug_assertions) {
                ui.add_space(ui.spacing().item_spacing.y);
                ui.group(|ui| {
                    ui.allocate_space(vec2(ui.available_width(), 0.));
                    self.debug_toggles(ui);
                });
            }
        });
    }

    fn tile_selector(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.strong("VRAM");
            ui.radio_value(&mut self.vram_selection_mode, VramSelectionMode::SingleTile, "8x8");
            ui.radio_value(&mut self.vram_selection_mode, VramSelectionMode::TwoByTwoTiles, "16x16");
        });
        Frame::canvas(ui.style()).show(ui, |ui| {
            let vram_renderer = Arc::clone(&self.vram_renderer);
            let gfx_bufs = self.gfx_bufs;
            ui.add(
                VramView::new(vram_renderer, gfx_bufs)
                    .viewed_tiles(ViewedVramTiles::SpritesOnly)
                    .selection(self.vram_selection_mode, &mut self.selected_vram_tile)
                    .zoom(2.),
            );
        });
    }

    fn tile_selection_preview(&mut self, ui: &mut Ui) {
        let vram_renderer = Arc::clone(&self.vram_renderer);
        let gfx_bufs = self.gfx_bufs;

        ui.strong("Selection preview");
        let px = self.pixels_per_point;
        let preview_size = tweak!(8.);
        let zoom = tweak!(8.);
        let (rect, _response) =
            ui.allocate_exact_size(OnCanvas::splat(preview_size).to_screen(px, zoom).0, Sense::hover());

        let screen_size = match self.vram_selection_mode {
            VramSelectionMode::SingleTile => rect.size() * px,
            VramSelectionMode::TwoByTwoTiles => rect.size() * px * 2.,
        };
        let offset = vec2(-(self.selected_vram_tile.0 as f32), -32. - self.selected_vram_tile.1 as f32) * zoom;

        ui.painter().add(PaintCallback {
            rect,
            callback: Arc::new(CallbackFn::new(move |_info, painter| {
                vram_renderer
                    .lock()
                    .expect("Cannot lock mutex on selected tile view's tile renderer")
                    .paint(painter.gl(), &TileUniforms { gfx_bufs, screen_size, offset, zoom });
            })),
        });
    }

    fn palette_row_selector(&mut self, ui: &mut Ui) {
        ui.strong("Palette");
        Frame::canvas(ui.style()).show(ui, |ui| {
            let size = vec2(tweak!(230.), tweak!(115.));
            let palette_view = PaletteView::new(Arc::clone(&self.palette_renderer), self.gfx_bufs.palette_buf, size)
                .viewed_rows(ViewedPalettes::SpritesOnly)
                .selection(SelectionType::Row(&mut self.selected_palette));
            if ui.add(palette_view).changed() {
                self.update_tile_palette();
            }
        });
    }

    #[cfg(debug_assertions)]
    fn debug_toggles(&mut self, ui: &mut Ui) {
        ui.collapsing("Debug", |ui| {
            ui.checkbox(&mut self.debug_selection_bounds, "Show selection bounds");
        });
    }
}
