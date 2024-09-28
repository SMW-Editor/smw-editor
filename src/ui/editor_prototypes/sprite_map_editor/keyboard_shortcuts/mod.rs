mod data;

pub(super) use data::*;
use egui::{vec2, Event, InputState, Key, Pos2, Ui};
use smwe_math::coordinates::{OnCanvas, OnScreen};
use smwe_render::tile_renderer::TileJson;

use super::UiSpriteMapEditor;
use crate::ui::editing_mode::{EditingMode, SnapToGrid};

impl UiSpriteMapEditor {
    pub(super) fn handle_input(&mut self, ui: &Ui) {
        ui.input_mut(|input| {
            if input.consume_shortcut(&SHORTCUT_NEW) {
                self.create_new_map();
            }
            if input.consume_shortcut(&SHORTCUT_SAVE) {
                self.save_map_dialog();
            }
            if input.consume_shortcut(&SHORTCUT_OPEN) {
                self.open_map_dialog();
            }
            if input.consume_shortcut(&SHORTCUT_UNDO) {
                self.handle_undo();
            }
            if input.consume_shortcut(&SHORTCUT_REDO) {
                self.handle_redo();
            }
            if input.consume_shortcut(&SHORTCUT_SELECT_ALL) {
                self.select_all_tiles();
            }
            if input.consume_shortcut(&SHORTCUT_UNSELECT_ALL) {
                self.unselect_all_tiles();
            }
            if input.consume_shortcut(&SHORTCUT_DELETE_SELECTED) {
                self.delete_selected_tiles();
            }
            self.kb_shortcut_move_selection(input);
            self.kb_shortcuts_tools(input);
            self.handle_zoom(input);
        });

        if ui.input(|input| input.events.contains(&Event::Copy)) {
            ui.output_mut(|output| self.handle_copy(output));
        }
        if ui.input(|input| input.events.contains(&Event::Cut)) {
            ui.output_mut(|output| self.handle_cut(output));
        }
    }

    pub(super) fn kb_shortcut_paste(&mut self, input: &InputState, canvas_top_left: OnScreen<Pos2>) {
        for event in input.events.iter() {
            if let Event::Paste(pasted_text) = event {
                if let Ok(pasted_tiles) = serde_json::from_str::<Vec<TileJson>>(pasted_text) {
                    let hover_pos = OnScreen(input.pointer.hover_pos().expect("Failed to get hover position"));
                    let paste_offset =
                        hover_pos.relative_to(canvas_top_left).to_vec2().to_canvas(self.pixels_per_point, self.zoom);
                    self.paste_tiles_to(pasted_tiles, paste_offset);
                }
                break;
            }
        }
    }

    fn kb_shortcut_move_selection(&mut self, input: &InputState) {
        let move_distance = if input.modifiers.shift_only() { self.tile_size_px } else { 1. };
        let moves = [
            (Key::ArrowUp, vec2(0., -move_distance)),
            (Key::ArrowDown, vec2(0., move_distance)),
            (Key::ArrowLeft, vec2(-move_distance, 0.)),
            (Key::ArrowRight, vec2(move_distance, 0.)),
        ];
        for (key, offset) in moves {
            if input.key_pressed(key) {
                self.move_selected_tiles_by(
                    OnCanvas(offset),
                    input.modifiers.shift_only().then_some(SnapToGrid::default()),
                );
            }
        }
    }

    fn kb_shortcuts_tools(&mut self, input: &mut InputState) {
        let modes = [
            (&SHORTCUT_MODE_INSERT, EditingMode::Move(None)),
            (&SHORTCUT_MODE_SELECT, EditingMode::Select),
            (&SHORTCUT_MODE_ERASE, EditingMode::Erase),
            (&SHORTCUT_MODE_PROBE, EditingMode::Probe),
            (&SHORTCUT_MODE_FLIP_HORIZONTALLY, EditingMode::FlipHorizontally),
            (&SHORTCUT_MODE_FLIP_VERTICALLY, EditingMode::FlipVertically),
        ];
        for (shortcut, mode) in modes {
            if input.consume_shortcut(shortcut) {
                self.editing_mode = mode;
                break;
            }
        }
    }

    fn handle_zoom(&mut self, input: &mut InputState) {
        if input.zoom_delta() > 1. || input.consume_shortcut(&SHORTCUT_ZOOM_IN) {
            self.zoom = Self::MAX_ZOOM.min(self.zoom + 0.25);
        } else if input.zoom_delta() < 1. || input.consume_shortcut(&SHORTCUT_ZOOM_OUT) {
            self.zoom = Self::MIN_ZOOM.max(self.zoom - 0.25);
        }
    }
}
