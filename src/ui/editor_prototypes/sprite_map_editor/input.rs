use duplicate::duplicate;
use egui::{vec2, Key, Ui};
use itertools::Itertools;

use crate::ui::{editing_mode::SnapToGrid, editor_prototypes::sprite_map_editor::UiSpriteMapEditor};

impl UiSpriteMapEditor {
    pub(super) fn handle_keyboard(&mut self, ui: &mut Ui) {
        ui.input(|input| {
            let move_distance = if input.modifiers.shift_only() { self.tile_size_px } else { 1. };

            // Select all
            if input.modifiers.command_only() && input.key_pressed(Key::A) {
                self.mark_tiles_as_selected(0..self.sprite_tiles.len());
            }

            // Unselect all
            if input.key_pressed(Key::Escape) {
                self.unselect_all_tiles();
            }

            // Delete
            if input.key_pressed(Key::Delete) {
                for idx in self.selected_sprite_tile_indices.drain().sorted().rev() {
                    self.sprite_tiles.remove(idx);
                }
                self.selection_bounds = None;
                self.upload_tiles();
            }

            // Move selection
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
        });
    }
}
