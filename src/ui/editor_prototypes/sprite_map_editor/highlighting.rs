use egui::*;

use super::{math::tile_contains_point, UiSpriteMapEditor};
use crate::ui::{
    editing_mode::*,
    style::{CellSelectorStyle, EditorStyle},
};

impl UiSpriteMapEditor {
    pub(super) fn higlight_hovered_tiles(&mut self, ui: &mut Ui, relative_pointer_pos: Pos2, canvas_left_top: Pos2) {
        let scaling_factor = self.zoom / self.pixels_per_point;
        match self.editing_mode {
            EditingMode::Move(_) => {
                if self
                    .selected_sprite_tile_indices
                    .iter()
                    .map(|&i| self.sprite_tiles[i])
                    .any(|tile| tile_contains_point(tile, relative_pointer_pos, scaling_factor))
                {
                    self.hovering_selected_tile = true;
                } else if let Some(hovered_tile) = self
                    .sprite_tiles
                    .iter()
                    .find(|&&tile| tile_contains_point(tile, relative_pointer_pos, scaling_factor))
                {
                    let exact_tile_pos = canvas_left_top + (hovered_tile.pos().to_vec2() * scaling_factor);
                    self.highlight_tile_at(
                        ui,
                        exact_tile_pos,
                        CellSelectorStyle::get_from_egui(ui.ctx(), |style| style.hover_highlight_color),
                    );
                }
            }
            EditingMode::Erase => {
                if let Some(hovered_tile) = self
                    .sprite_tiles
                    .iter()
                    .find(|&&tile| tile_contains_point(tile, relative_pointer_pos, scaling_factor))
                {
                    self.highlight_tile_at(
                        ui,
                        ((hovered_tile.pos().to_vec2() * scaling_factor) + canvas_left_top.to_vec2()).to_pos2(),
                        CellSelectorStyle::get_from_egui(ui.ctx(), |style| style.delete_highlight_color),
                    );
                }
            }
            EditingMode::Draw => {
                let scale_pp = self.tile_size_px / self.pixels_per_point;
                let hovered_tile = (relative_pointer_pos.to_vec2() / scale_pp / self.zoom).floor();
                let hovered_tile = hovered_tile.clamp(vec2(0., 0.), vec2(31., 31.));
                let hovered_tile_exact_offset = hovered_tile * scale_pp * self.zoom;
                self.highlight_tile_at(
                    ui,
                    canvas_left_top + hovered_tile_exact_offset,
                    CellSelectorStyle::get_from_egui(ui.ctx(), |style| style.hover_highlight_color),
                );
            }
            _ => {}
        }
    }

    pub(super) fn highlight_tile_at(&self, ui: &mut Ui, pos: Pos2, color: impl Into<Color32>) {
        ui.painter().rect_filled(
            Rect::from_min_size(pos, Vec2::splat(self.tile_size_px * self.zoom / self.pixels_per_point)),
            Rounding::none(),
            color,
        );
    }

    pub(super) fn highlight_selected_tiles(&mut self, ui: &mut Ui, canvas_pos: Pos2) {
        let selection_offset = self.selection_offset.take().unwrap_or_default();
        for tile in self.selected_sprite_tile_indices.iter().map(|&idx| &self.sprite_tiles[idx]) {
            self.highlight_tile_at(
                ui,
                canvas_pos + selection_offset + tile.pos().to_vec2() / self.pixels_per_point * self.zoom,
                CellSelectorStyle::get_from_egui(ui.ctx(), |style| {
                    if self.hovering_selected_tile {
                        style.hover_highlight_color
                    } else {
                        style.selection_highlight_color
                    }
                }),
            );
        }
    }
}
