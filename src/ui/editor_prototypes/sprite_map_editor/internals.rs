use duplicate::duplicate;
use egui::emath::*;
use itertools::Itertools;
use num::Integer;
use paste::paste;
use smwe_emu::{emu::CheckedMem, Cpu};
use smwe_render::tile_renderer::Tile;

use crate::ui::{
    editing_mode::{Drag, Selection, SnapToGrid},
    editor_prototypes::sprite_map_editor::UiSpriteMapEditor,
    EditorState,
};

impl UiSpriteMapEditor {
    pub(super) fn update_cpu(&mut self, state: &mut EditorState) {
        let project = state.project.as_ref().unwrap().borrow();
        let mut cpu = Cpu::new(CheckedMem::new(project.rom.clone()));
        drop(project);
        smwe_emu::emu::decompress_sublevel(&mut cpu, self.level_num);
        println!("Updated CPU");
        state.cpu = Some(cpu);
    }

    pub(super) fn update_renderers(&mut self, state: &mut EditorState) {
        let cpu = state.cpu.as_mut().unwrap();
        self.gfx_bufs.upload_palette(&self.gl, &cpu.mem.cgram);
        self.gfx_bufs.upload_vram(&self.gl, &cpu.mem.vram);
    }

    pub(super) fn upload_tiles(&self) {
        self.sprite_renderer
            .lock()
            .expect("Cannot lock mutex on sprite renderer")
            .set_tiles(&self.gl, self.sprite_tiles.clone());
    }

    pub(super) fn handle_edition_insert(&mut self, grid_cell_pos: Pos2) {
        if self.last_inserted_tile != grid_cell_pos {
            self.add_selected_tile_at(grid_cell_pos);
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
        if self.any_tile_contains_pointer(drag_data.from, canvas_top_left_pos) {
            self.move_selected_tiles_by(
                drag_data.delta() / self.zoom * self.pixels_per_point,
                snap_to_grid.then(|| {
                    let scale_pp = self.scale / self.pixels_per_point;
                    let pointer_in_canvas = (drag_data.to - canvas_top_left_pos).to_pos2();
                    let hovered_tile = point_to_tile_coords(pointer_in_canvas, scale_pp, self.zoom);
                    let hovered_tile_exact_offset = hovered_tile * scale_pp * self.zoom;
                    let cell_origin = (pointer_in_canvas - hovered_tile_exact_offset).to_vec2() / self.zoom;
                    SnapToGrid { cell_origin }
                }),
            );
        }
    }

    pub(super) fn handle_edition_dragging(
        &mut self, mut drag_data: Drag, snap_to_grid: bool, canvas_top_left_pos: Pos2,
    ) {
        if self.any_tile_contains_pointer(drag_data.from, canvas_top_left_pos) {
            if snap_to_grid {
                let scale_pp = self.scale / self.pixels_per_point;
                let bounds = self.selection_bounds.expect("unset even though some tiles are selected");

                let bounds_min_at_grid =
                    point_to_tile_coords((bounds.min.to_vec2() * scale_pp * self.zoom).to_pos2(), scale_pp, self.zoom);
                let started_tile =
                    point_to_tile_coords((drag_data.from - canvas_top_left_pos).to_pos2(), scale_pp, self.zoom);
                let hovered_tile =
                    point_to_tile_coords((drag_data.to - canvas_top_left_pos).to_pos2(), scale_pp, self.zoom);

                let bounds_offset = bounds.min - bounds_min_at_grid.to_pos2();
                let started_tile_exact_offset = started_tile * scale_pp * self.zoom;
                let hovered_tile_exact_offset = hovered_tile * scale_pp * self.zoom;

                drag_data.from = canvas_top_left_pos + started_tile_exact_offset + bounds_offset;
                drag_data.to = canvas_top_left_pos + hovered_tile_exact_offset;

                // move_offset.x = move_offset.x.clamp(-bounds.min.x, (31. * self.scale) - bounds.max.x);
                // move_offset.y = move_offset.y.clamp(-bounds.min.y, (31. * self.scale) - bounds.max.y);
            }
            self.selection_offset = Some(drag_data.delta());
        }
    }

    pub(super) fn any_tile_contains_pointer(&mut self, pointer_pos: Pos2, canvas_top_left_pos: Pos2) -> bool {
        let tile_contains_pointer = |tile| {
            tile_contains_point(tile, (pointer_pos - canvas_top_left_pos).to_pos2(), self.zoom / self.pixels_per_point)
        };
        self.selected_sprite_tile_indices.iter().map(|&i| &self.sprite_tiles[i]).any(tile_contains_pointer)
    }

    pub(super) fn handle_edition_erase(&mut self, relative_pointer_pos: Pos2) {
        self.delete_tiles_at(relative_pointer_pos);
        self.unselect_all_tiles();
    }

    pub(super) fn handle_edition_probe(&mut self, relative_pointer_pos: Pos2) {
        self.probe_tile_at(relative_pointer_pos);
        self.unselect_all_tiles();
    }

    pub(super) fn move_selected_tiles_by(&mut self, mut move_offset: Vec2, snap_to_grid: Option<SnapToGrid>) {
        if self.selected_sprite_tile_indices.is_empty() {
            return;
        }

        let bounds = self.selection_bounds.expect("unset even though some tiles are selected");
        move_offset.x = move_offset.x.clamp(-bounds.min.x, (31. * self.scale) - bounds.max.x);
        move_offset.y = move_offset.y.clamp(-bounds.min.y, (31. * self.scale) - bounds.max.y);

        for &idx in self.selected_sprite_tile_indices.iter() {
            self.sprite_tiles[idx].move_by(move_offset);
            if let Some(snap_to_grid) = snap_to_grid {
                self.sprite_tiles[idx].snap_to_grid(self.scale as u32, snap_to_grid.cell_origin);
            }
        }

        self.compute_selection_bounds();
        self.upload_tiles();
    }

    pub(super) fn add_selected_tile_at(&mut self, pos: Pos2) {
        let tile_idx = (self.selected_vram_tile.0 + self.selected_vram_tile.1 * 16) as usize;
        let mut tile = self.tile_palette[tile_idx + (32 * 16)];
        tile.0[0] = pos.x.floor() as u32;
        tile.0[1] = pos.y.floor() as u32;
        self.sprite_tiles.push(tile);
        self.upload_tiles();
    }

    pub(super) fn select_tile_at(&mut self, pos: Pos2, clear_previous_selection: bool) {
        if clear_previous_selection {
            self.unselect_all_tiles();
        }
        if let Some((idx, _)) = self
            .sprite_tiles
            .iter()
            .enumerate()
            .rev()
            .find(|(_, tile)| tile_contains_point(tile, pos, self.zoom / self.pixels_per_point))
        {
            self.selected_sprite_tile_indices.insert(idx);
        }
    }

    pub(super) fn select_tiles_in(&mut self, rect: Rect, clear_previous_selection: bool) {
        if clear_previous_selection {
            self.unselect_all_tiles();
        }
        let indices = self
            .sprite_tiles
            .iter()
            .enumerate()
            .filter(|(_, tile)| tile_intersects_rect(tile, rect, self.zoom / self.pixels_per_point))
            .map(|(i, _)| i)
            .collect_vec();
        self.mark_tiles_as_selected(indices.into_iter());
    }

    pub(super) fn unselect_all_tiles(&mut self) {
        self.selected_sprite_tile_indices.clear();
        self.selection_bounds = None;
    }

    pub(super) fn mark_tiles_as_selected(&mut self, indices: impl IntoIterator<Item = usize>) {
        for index in indices {
            self.selected_sprite_tile_indices.insert(index);
        }
        self.compute_selection_bounds();
    }

    pub(super) fn compute_selection_bounds(&mut self) {
        self.selection_bounds = (!self.selected_sprite_tile_indices.is_empty()).then(|| {
            duplicate! {
                [dimension; [x]; [y]]
                paste! {
                    let ([<min_tile_ dimension>], [<max_tile_ dimension>]) = self
                        .selected_sprite_tile_indices
                        .iter()
                        .map(|&i| self.sprite_tiles[i].pos())
                        .minmax_by(|a, b| a.dimension.total_cmp(&b.dimension))
                        .into_option()
                        .map(|(min, max)| (min.dimension, max.dimension))
                        .unwrap();
                }
            }
            Rect::from_min_max(pos2(min_tile_x, min_tile_y), pos2(max_tile_x, max_tile_y))
        });
    }

    pub(super) fn delete_tiles_at(&mut self, pos: Pos2) {
        self.sprite_tiles.retain(|tile| !tile_contains_point(tile, pos, self.zoom / self.pixels_per_point));
        self.upload_tiles();
    }

    pub(super) fn probe_tile_at(&mut self, pos: Pos2) {
        if let Some(tile) = self
            .sprite_tiles
            .iter()
            .rev()
            .find(|tile| tile_contains_point(tile, pos, self.zoom / self.pixels_per_point))
        {
            let (y, x) = tile.tile_num().div_rem(&16);
            self.selected_vram_tile = (x, y - 96);
        };
    }
}

pub(super) fn tile_contains_point(tile: &Tile, point: Pos2, scale: f32) -> bool {
    Rect::from_min_size((tile.pos().to_vec2() * scale).to_pos2(), Vec2::splat(tile.scale() as f32 * scale))
        .contains(point)
}

pub(super) fn tile_intersects_rect(tile: &Tile, rect: Rect, scale: f32) -> bool {
    Rect::from_min_size((tile.pos().to_vec2() * scale).to_pos2(), Vec2::splat(tile.scale() as f32 * scale))
        .intersects(rect)
}

pub(super) fn point_to_tile_coords(pos_in_canvas: Pos2, scale_pp: f32, zoom: f32) -> Vec2 {
    (pos_in_canvas.to_vec2() / scale_pp / zoom).floor().clamp(vec2(0., 0.), vec2(31., 31.))
}
