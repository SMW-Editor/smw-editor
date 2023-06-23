use egui::{Pos2, Vec2};
use smwe_math::coordinates::{OnCanvas, OnGrid, OnScreen};
use smwe_widgets::vram_view::VramSelectionMode;

use super::UiSpriteMapEditor;
use crate::ui::editing_mode::{Drag, FlipDirection, Selection, SnapToGrid};

impl UiSpriteMapEditor {
    pub(super) fn handle_edition_insert(&mut self, grid_cell_pos: OnCanvas<Pos2>) {
        if self.last_inserted_tile != grid_cell_pos {
            match self.vram_selection_mode {
                VramSelectionMode::SingleTile => self.add_selected_tile_at(grid_cell_pos),
                VramSelectionMode::TwoByTwoTiles => {
                    let current_selection = self.selected_vram_tile;
                    for offset in [(0, 0), (0, 1), (1, 0), (1, 1)] {
                        self.selected_vram_tile.0 = current_selection.0 + offset.0;
                        self.selected_vram_tile.1 = current_selection.1 + offset.1;
                        let offset = OnGrid::<Vec2>::new(offset.0 as f32, offset.1 as f32).to_canvas(self.tile_size_px);
                        let pos = OnCanvas(grid_cell_pos.0 + offset.0);
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
        &mut self, selection: Selection, clear_previous_selection: bool, canvas_top_left_pos: OnScreen<Pos2>,
    ) {
        match selection {
            Selection::Click(Some(origin)) => {
                let pos = origin.0 - canvas_top_left_pos.0;
                self.select_tile_at(OnScreen(pos.to_pos2()), clear_previous_selection);
            }
            Selection::Drag(Some(selection_rect)) => {
                self.select_tiles_in(
                    OnScreen(selection_rect.0.translate(-canvas_top_left_pos.0.to_vec2())),
                    clear_previous_selection,
                );
            }
            _ => {}
        }
    }

    pub(super) fn handle_edition_drop(
        &mut self, drag_data: Drag, snap_to_grid: bool, canvas_top_left_pos: OnScreen<Pos2>,
    ) {
        if !self.any_tile_contains_pointer(drag_data.from, canvas_top_left_pos) {
            return;
        }

        self.move_selected_tiles_by(
            drag_data.delta().to_canvas(self.pixels_per_point, self.zoom),
            snap_to_grid.then(|| {
                let pointer_in_canvas = drag_data.from.relative_to(canvas_top_left_pos);
                let hovered_tile_exact_offset = pointer_in_canvas
                    .to_grid(self.pixels_per_point, self.zoom, self.tile_size_px)
                    .clamp(OnGrid::<Pos2>::ZERO, self.grid_size.to_pos2())
                    .to_screen(self.pixels_per_point, self.zoom, self.tile_size_px);
                let cell_origin = pointer_in_canvas.relative_to(hovered_tile_exact_offset).to_vec2() / self.zoom;
                SnapToGrid { cell_origin }
            }),
        );
    }

    pub(super) fn handle_edition_dragging(
        &mut self, mut drag_data: Drag, snap_to_grid: bool, canvas_top_left_pos: OnScreen<Pos2>,
    ) {
        if !self.any_tile_contains_pointer(drag_data.from, canvas_top_left_pos) {
            return;
        }

        if snap_to_grid {
            let sel_bounds = self.selection_bounds.expect("unset even though some tiles are selected");

            let bounds_min_grid = sel_bounds.left_top().to_grid(self.tile_size_px);
            let started_tile = drag_data.from.relative_to(canvas_top_left_pos).to_grid(
                self.pixels_per_point,
                self.zoom,
                self.tile_size_px,
            );
            let hovered_tile = drag_data.to.relative_to(canvas_top_left_pos).to_grid(
                self.pixels_per_point,
                self.zoom,
                self.tile_size_px,
            );

            let bounds_at_grid_exact_offset =
                bounds_min_grid.to_screen(self.pixels_per_point, self.zoom, self.tile_size_px).to_vec2();
            let started_tile_exact_offset =
                started_tile.to_screen(self.pixels_per_point, self.zoom, self.tile_size_px).to_vec2();
            let hovered_tile_exact_offset =
                hovered_tile.to_screen(self.pixels_per_point, self.zoom, self.tile_size_px).to_vec2();

            let bounds_screen = sel_bounds.left_top().to_screen(self.pixels_per_point, self.zoom);
            let bounds_offset = bounds_screen.to_vec2() - bounds_at_grid_exact_offset;
            drag_data.from = canvas_top_left_pos + started_tile_exact_offset + bounds_offset;
            drag_data.to = canvas_top_left_pos + hovered_tile_exact_offset;
        }

        // todo restrict moving selection display to canvas
        // move_offset.x = move_offset.x.clamp(-bounds.min.x, (31. * self.scale) - bounds.max.x);
        // move_offset.y = move_offset.y.clamp(-bounds.min.y, (31. * self.scale) - bounds.max.y);

        self.selection_offset = Some(drag_data.delta());
    }

    pub(super) fn handle_edition_erase(&mut self, relative_pointer_pos: OnScreen<Pos2>) {
        self.delete_tiles_at(relative_pointer_pos);
        self.unselect_all_tiles();
    }

    pub(super) fn handle_edition_probe(&mut self, relative_pointer_pos: OnScreen<Pos2>) {
        self.probe_tile_at(relative_pointer_pos);
        self.unselect_all_tiles();
    }

    pub(super) fn handle_edition_flip(&mut self, relative_pointer_pos: OnScreen<Pos2>, flip_direction: FlipDirection) {
        let pointer_on_canvas = relative_pointer_pos.to_canvas(self.pixels_per_point, self.zoom);
        if self.any_selected_tile_contains_point(pointer_on_canvas) {
            self.flip_selected_tiles(flip_direction);
        } else if let Some(tile) = self.find_tile_containing_point_mut(pointer_on_canvas) {
            match flip_direction {
                FlipDirection::Horizontal => tile.toggle_flip_x(),
                FlipDirection::Vertical => tile.toggle_flip_y(),
            }
            self.upload_tiles();
        }
    }
}
