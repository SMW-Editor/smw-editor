use duplicate::duplicate;
use egui::{vec2, Event, Key, KeyboardShortcut, Modifiers, Pos2, Ui};
use smwe_math::coordinates::{OnCanvas, OnScreen};
use smwe_render::tile_renderer::TileJson;

use super::UiSpriteMapEditor;
use crate::ui::editing_mode::SnapToGrid;

const SHORTCUT_SELECT_ALL: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::A);
const SHORTCUT_ZOOM_IN: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::PlusEquals);
const SHORTCUT_ZOOM_OUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Minus);

impl UiSpriteMapEditor {
    pub(super) fn handle_input(&mut self, ui: &mut Ui) {
        self.kb_shortcut_select_all(ui);
        self.kb_shortcut_unselect_all(ui);
        self.kb_shortcut_delete_selection(ui);
        self.kb_shortcut_move_selection(ui);
        self.handle_zoom(ui);
        self.handle_copy(ui);
        self.handle_cut(ui);
    }

    fn kb_shortcut_select_all(&mut self, ui: &mut Ui) {
        if ui.input_mut(|input| input.consume_shortcut(&SHORTCUT_SELECT_ALL)) {
            self.mark_tiles_as_selected(0..self.sprite_tiles.len());
        }
    }

    fn kb_shortcut_unselect_all(&mut self, ui: &mut Ui) {
        if ui.input(|input| input.key_pressed(Key::Escape)) {
            self.unselect_all_tiles();
        }
    }

    fn kb_shortcut_delete_selection(&mut self, ui: &mut Ui) {
        if ui.input(|input| input.key_pressed(Key::Delete)) {
            self.delete_selected_tiles();
        }
    }

    fn kb_shortcut_move_selection(&mut self, ui: &mut Ui) {
        let move_distance = if ui.input(|input| input.modifiers.shift_only()) { self.tile_size_px } else { 1. };
        duplicate! {
            [
                key          offset;
                [ArrowUp]    [vec2(0., -move_distance)];
                [ArrowDown]  [vec2(0.,  move_distance)];
                [ArrowLeft]  [vec2(-move_distance, 0.)];
                [ArrowRight] [vec2( move_distance, 0.)];
            ]
            if ui.input(|input| input.key_pressed(Key::key)) {
                self.move_selected_tiles_by(
                    OnCanvas(offset),
                    ui.input(|input| input.modifiers.shift_only().then_some(SnapToGrid::default())),
                );
            }
        }
    }

    fn handle_zoom(&mut self, ui: &mut Ui) {
        if ui.input_mut(|input| input.zoom_delta() > 1. || input.consume_shortcut(&SHORTCUT_ZOOM_IN)) {
            self.zoom = 4.0f32.min(self.zoom + 0.25);
        } else if ui.input_mut(|input| input.zoom_delta() < 1. || input.consume_shortcut(&SHORTCUT_ZOOM_OUT)) {
            self.zoom = 1.0f32.max(self.zoom - 0.25);
        }
    }

    fn handle_copy(&mut self, ui: &mut Ui) {
        if ui.input(|input| input.events.contains(&Event::Copy)) {
            ui.output_mut(|output| self.copy_selected_tiles(output));
        }
    }

    fn handle_cut(&mut self, ui: &mut Ui) {
        if ui.input(|input| input.events.contains(&Event::Cut)) {
            ui.output_mut(|output| self.copy_selected_tiles(output));
            self.delete_selected_tiles();
        }
    }

    pub(super) fn handle_paste(&mut self, ui: &mut Ui, canvas_top_left: OnScreen<Pos2>) {
        ui.input(|input| {
            for event in input.events.iter() {
                if let Event::Paste(pasted_text) = event {
                    if let Ok(pasted_tiles) = serde_json::from_str::<Vec<TileJson>>(pasted_text) {
                        let pointer_pos = OnScreen(input.pointer.hover_pos().expect("Failed to get hover position"))
                            - canvas_top_left;
                        let paste_offset = pointer_pos.to_canvas(self.pixels_per_point, self.zoom);
                        self.paste_tiles_at(pasted_tiles, paste_offset);
                    }
                    break;
                }
            }
        });
    }
}
