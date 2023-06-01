use egui::{vec2, Pos2};
use smwe_widgets::vram_view::VramSelectionMode;

use super::{math::point_to_tile_coords, UiSpriteMapEditor};
use crate::ui::editing_mode::{Drag, Selection, SnapToGrid};

impl UiSpriteMapEditor {
    pub(super) fn handle_edition_insert(&mut self, grid_cell_pos: Pos2) {
        if self.last_inserted_tile != grid_cell_pos {
            match self.vram_selection_mode {
                VramSelectionMode::SingleTile => self.add_selected_tile_at(grid_cell_pos),
                VramSelectionMode::TwoByTwoTiles => {
                    let current_selection = self.selected_vram_tile;
                    for offset in [(0, 0), (0, 1), (1, 0), (1, 1)] {
                        self.selected_vram_tile.0 = current_selection.0 + offset.0;
                        self.selected_vram_tile.1 = current_selection.1 + offset.1;
                        let pos = grid_cell_pos + (self.tile_size_px * vec2(offset.0 as f32, offset.1 as f32));
                        self.add_selected_tile_at(pos);
                    }
                    self.selected_vram_tile = current_selection;
                }
            }
            self.last_inserted_tile = grid_cell_pos;
        }
        self.unselect_all_tiles();
    }

    pub(super) fn handle_selection_plot(
        &mut self, selection: Selection, clear_previous_selection: bool, canvas_top_left_pos: Pos2,
    ) {
        match selection {
            Selection::Click(Some(origin)) => {
                let pos = origin - canvas_top_left_pos;
                self.select_tile_at(pos.to_pos2(), clear_previous_selection);
            }
            Selection::Drag(Some(selection_rect)) => {
                self.select_tiles_in(
                    selection_rect.translate(-canvas_top_left_pos.to_vec2()),
                    clear_previous_selection,
                );
            }
            _ => {}
        }
    }

    pub(super) fn handle_edition_drop_moved(&mut self, drag_data: Drag, snap_to_grid: bool, canvas_top_left_pos: Pos2) {
        if !self.any_tile_contains_pointer(drag_data.from, canvas_top_left_pos) {
            return;
        }

        self.move_selected_tiles_by(
            drag_data.delta() / self.zoom * self.pixels_per_point,
            snap_to_grid.then(|| {
                let scale_pp = self.tile_size_px / self.pixels_per_point;
                let pointer_in_canvas = (drag_data.from - canvas_top_left_pos).to_pos2();
                let hovered_tile = point_to_tile_coords(pointer_in_canvas, scale_pp, self.zoom);
                let hovered_tile_exact_offset = hovered_tile * scale_pp * self.zoom;
                let cell_origin = (pointer_in_canvas - hovered_tile_exact_offset).to_vec2() / self.zoom;
                SnapToGrid { cell_origin }
            }),
        );
    }

    pub(super) fn handle_edition_dragging(
        &mut self, mut drag_data: Drag, snap_to_grid: bool, canvas_top_left_pos: Pos2,
    ) {
        if !self.any_tile_contains_pointer(drag_data.from, canvas_top_left_pos) {
            return;
        }

        if snap_to_grid {
            let scale_grid = self.tile_size_px / self.pixels_per_point;
            let sel_bounds_scaling = self.zoom / self.pixels_per_point;
            let sel_bounds = self.selection_bounds.expect("unset even though some tiles are selected");

            let bounds_min_at_grid =
                point_to_tile_coords((sel_bounds.min.to_vec2() * sel_bounds_scaling).to_pos2(), scale_grid, self.zoom);
            let started_tile =
                point_to_tile_coords((drag_data.from - canvas_top_left_pos).to_pos2(), scale_grid, self.zoom);
            let hovered_tile =
                point_to_tile_coords((drag_data.to - canvas_top_left_pos).to_pos2(), scale_grid, self.zoom);

            let bounds_at_grid_exact_offset = bounds_min_at_grid * scale_grid * self.zoom;
            let started_tile_exact_offset = started_tile * scale_grid * self.zoom;
            let hovered_tile_exact_offset = hovered_tile * scale_grid * self.zoom;

            let bounds_offset = (sel_bounds.min.to_vec2() * sel_bounds_scaling) - bounds_at_grid_exact_offset;
            drag_data.from = canvas_top_left_pos + started_tile_exact_offset + bounds_offset;
            drag_data.to = canvas_top_left_pos + hovered_tile_exact_offset;
        }

        // todo restrict moving selection display to canvas
        // move_offset.x = move_offset.x.clamp(-bounds.min.x, (31. * self.scale) - bounds.max.x);
        // move_offset.y = move_offset.y.clamp(-bounds.min.y, (31. * self.scale) - bounds.max.y);

        self.selection_offset = Some(drag_data.delta());
    }

    pub(super) fn handle_edition_erase(&mut self, relative_pointer_pos: Pos2) {
        self.delete_tiles_at(relative_pointer_pos);
        self.unselect_all_tiles();
    }

    pub(super) fn handle_edition_probe(&mut self, relative_pointer_pos: Pos2) {
        self.probe_tile_at(relative_pointer_pos);
        self.unselect_all_tiles();
    }
}
