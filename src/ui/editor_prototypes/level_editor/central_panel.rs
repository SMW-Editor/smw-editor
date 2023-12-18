use std::sync::Arc;

use egui::{vec2, Color32, PaintCallback, PointerButton, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2};
use egui_glow::CallbackFn;
use inline_tweak::tweak;
use smwe_math::coordinates::{OnCanvas, OnGrid, OnScreen};
use smwe_render::color::Abgr1555;

use super::UiLevelEditor;

impl UiLevelEditor {
    fn editable_area_rect(&self, view_rect: OnScreen<Rect>) -> OnScreen<Rect> {
        let (level_width, level_height) = self.level_properties.level_dimensions_in_tiles();
        let editable_area_size = OnGrid::<Vec2>::new(level_width as f32, level_height as f32).to_screen(
            self.pixels_per_point,
            self.zoom,
            self.tile_size_px,
        );
        let editable_area_min = OnCanvas(self.offset).to_screen(self.pixels_per_point, self.zoom).to_pos2()
            + OnScreen(view_rect.min).to_vec2();
        OnScreen::<Rect>::from_min_size(editable_area_min, editable_area_size).intersect(view_rect)
    }

    pub(super) fn central_panel(&mut self, ui: &mut Ui) {
        let level_renderer = Arc::clone(&self.level_renderer);
        let (view_rect, response) =
            ui.allocate_exact_size(vec2(ui.available_width(), ui.available_height()), Sense::click_and_drag());
        let screen_size_px = view_rect.size() * ui.ctx().pixels_per_point();

        let zoom = self.zoom;
        if response.dragged_by(PointerButton::Middle) {
            let mut level_renderer = level_renderer.lock().unwrap();
            let delta = response.drag_delta();
            self.offset += delta / zoom;
            level_renderer.set_offset(self.offset);
        }

        // Background.
        let bg_color = self.cpu.mem.load_u16(0x7E0701);
        let bg_color = Color32::from(Abgr1555(bg_color));
        ui.painter().rect_filled(self.editable_area_rect(OnScreen(view_rect)).0, Rounding::ZERO, bg_color);

        // Level.
        ui.painter().add(PaintCallback {
            rect:     view_rect,
            callback: Arc::new(CallbackFn::new(move |_info, painter| {
                level_renderer.lock().expect("Cannot lock mutex on level_renderer").paint(
                    painter.gl(),
                    screen_size_px,
                    zoom,
                );
            })),
        });

        // Grid.
        if self.always_show_grid || ui.input(|i| i.modifiers.shift_only()) {
            let spacing = self.zoom * self.tile_size_px / self.pixels_per_point;
            let stroke = Stroke::new(1., Color32::from_white_alpha(tweak!(70)));
            for col in 0..=(view_rect.width() / spacing) as u32 {
                let x_offset = (self.offset.x * self.zoom / self.pixels_per_point).rem_euclid(spacing);
                let x_coord = col as f32 * spacing + x_offset;
                ui.painter().vline(view_rect.min.x + x_coord, view_rect.min.y..=view_rect.max.y, stroke);
            }
            for row in 0..=(view_rect.height() / spacing) as u32 {
                let y_offset = (self.offset.y * self.zoom / self.pixels_per_point).rem_euclid(spacing);
                let y_coord = row as f32 * spacing + y_offset;
                ui.painter().hline(view_rect.min.x..=view_rect.max.x, view_rect.min.y + y_coord, stroke);
            }
        }
    }
}
