use duplicate::duplicate;
use egui::{vec2, InputState, Key, Ui};
use itertools::Itertools;

use crate::ui::{editing_mode::SnapToGrid, editor_prototypes::sprite_map_editor::UiSpriteMapEditor};

impl UiSpriteMapEditor {
    pub(super) fn handle_input(&mut self, ui: &mut Ui) {
        ui.input(|input| {
            self.kb_shortcut_select_all(input);
            self.kb_shortcut_unselect_all(input);
            self.kb_shortcut_delete_selection(input);
            self.kb_shortcut_move_selection(input);
            self.handle_zoom(input);
        });
    }

    fn kb_shortcut_select_all(&mut self, input: &InputState) {
        if input.modifiers.command_only() && input.key_pressed(Key::A) {
            self.mark_tiles_as_selected(0..self.sprite_tiles.len());
        }
    }

    fn kb_shortcut_unselect_all(&mut self, input: &InputState) {
        if input.key_pressed(Key::Escape) {
            self.unselect_all_tiles();
        }
    }

    fn kb_shortcut_delete_selection(&mut self, input: &InputState) {
        if input.key_pressed(Key::Delete) {
            for idx in self.selected_sprite_tile_indices.drain().sorted().rev() {
                self.sprite_tiles.remove(idx);
            }
            self.selection_bounds = None;
            self.upload_tiles();
        }
    }

    fn kb_shortcut_move_selection(&mut self, input: &InputState) {
        let move_distance = if input.modifiers.shift_only() { self.tile_size_px } else { 1. };
        duplicate! {
            [
                key          offset;
                [ArrowUp]    [vec2(0., -move_distance)];
                [ArrowDown]  [vec2(0.,  move_distance)];
                [ArrowLeft]  [vec2(-move_distance, 0.)];
                [ArrowRight] [vec2( move_distance, 0.)];
            ]
            if input.key_pressed(Key::key) {
                self.move_selected_tiles_by(
                    offset,
                    input.modifiers.shift_only().then_some(SnapToGrid::default()),
                );
            }
        }
    }

    fn handle_zoom(&mut self, input: &InputState) {
        if input.zoom_delta() > 1. || (input.modifiers.command_only() && input.key_pressed(Key::PlusEquals)) {
            self.zoom = 4.0f32.min(self.zoom + 0.25);
        } else if input.zoom_delta() < 1. || (input.modifiers.command_only() && input.key_pressed(Key::Minus)) {
            self.zoom = 1.0f32.max(self.zoom - 0.25);
        }
    }
}
