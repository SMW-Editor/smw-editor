use std::sync::Arc;

use eframe::{
    emath::vec2,
    epaint::{Color32, PaintCallback, Stroke},
};
use egui::{PointerButton, Sense, Ui};
use egui_glow::CallbackFn;
use inline_tweak::tweak;

use super::UiLevelEditor;

impl UiLevelEditor {
    pub(super) fn central_panel(&mut self, ui: &mut Ui) {
        let level_renderer = Arc::clone(&self.level_renderer);
        let (rect, response) =
            ui.allocate_exact_size(vec2(ui.available_width(), ui.available_height()), Sense::click_and_drag());
        let screen_size = rect.size() * ui.ctx().pixels_per_point();

        let zoom = self.zoom;
        if response.dragged_by(PointerButton::Middle) {
            let mut r = level_renderer.lock().unwrap();
            let delta = response.drag_delta();
            self.offset += delta / zoom;
            r.set_offset(self.offset);
        }

        ui.painter().add(PaintCallback {
            rect,
            callback: Arc::new(CallbackFn::new(move |_info, painter| {
                level_renderer.lock().expect("Cannot lock mutex on level_renderer").paint(
                    painter.gl(),
                    screen_size,
                    zoom,
                );
            })),
        });

        if self.always_show_grid || ui.input(|i| i.modifiers.shift_only()) {
            let spacing = self.zoom * self.tile_size_px / self.pixels_per_point;
            let stroke = Stroke::new(1., Color32::from_white_alpha(tweak!(70)));
            for col in 0..=(rect.width() / spacing) as u32 {
                let x_offset = (self.offset.x * self.zoom / self.pixels_per_point).rem_euclid(spacing);
                let x_coord = col as f32 * spacing + x_offset;
                ui.painter().vline(rect.min.x + x_coord, rect.min.y..=rect.max.y, stroke);
            }
            for row in 0..=(rect.height() / spacing) as u32 {
                let y_offset = (self.offset.y * self.zoom / self.pixels_per_point).rem_euclid(spacing);
                let y_coord = row as f32 * spacing + y_offset;
                ui.painter().hline(rect.min.x..=rect.max.x, rect.min.y + y_coord, stroke);
            }
        }
    }
}
