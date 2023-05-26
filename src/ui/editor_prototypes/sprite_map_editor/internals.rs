use duplicate::duplicate;
use egui::emath::*;
use itertools::Itertools;
use num::Integer;
use paste::paste;
use smwe_emu::{emu::CheckedMem, Cpu};
use smwe_render::tile_renderer::Tile;

use crate::ui::{
    editing_mode::{Drag, Selection},
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
        self.selected_sprite_tile_indices.clear();
    }

    pub(super) fn handle_selection_drag(
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
        let tile_contains_pointer = |tile| {
            tile_contains_point(
                tile,
                (drag_data.from - canvas_top_left_pos).to_pos2(),
                self.zoom / self.pixels_per_point,
            )
        };
        if self.selected_sprite_tile_indices.iter().map(|&i| &self.sprite_tiles[i]).any(tile_contains_pointer) {
            self.move_selected_tiles_by(drag_data.delta() / self.zoom * self.pixels_per_point, snap_to_grid);
        }
    }

    pub(super) fn handle_edition_moving(&mut self, drag_data: Drag, snap_to_grid: bool, canvas_top_left_pos: Pos2) {
        // todo highlight moved tiles
    }

    pub(super) fn handle_edition_erase(&mut self, relative_pointer_pos: Pos2) {
        self.delete_tiles_at(relative_pointer_pos);
        self.selected_sprite_tile_indices.clear();
    }

    pub(super) fn handle_edition_probe(&mut self, relative_pointer_pos: Pos2) {
        self.probe_tile_at(relative_pointer_pos);
        self.selected_sprite_tile_indices.clear();
    }

    pub(super) fn move_selected_tiles_by(&mut self, mut offset: Vec2, snap_to_grid: bool) {
        if self.selected_sprite_tile_indices.is_empty() {
            return;
        }

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

        offset.x = offset.x.clamp(-min_tile_x, (31. * self.scale) - max_tile_x);
        offset.y = offset.y.clamp(-min_tile_y, (31. * self.scale) - max_tile_y);

        for &idx in self.selected_sprite_tile_indices.iter() {
            self.sprite_tiles[idx].move_by(offset);
            if snap_to_grid {
                self.sprite_tiles[idx].snap_to_grid(self.scale as u32);
            }
        }

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
            self.selected_sprite_tile_indices.clear();
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
            self.selected_sprite_tile_indices.clear();
        }
        for (idx, _) in self
            .sprite_tiles
            .iter()
            .enumerate()
            .filter(|(_, tile)| tile_intersects_rect(tile, rect, self.zoom / self.pixels_per_point))
        {
            self.selected_sprite_tile_indices.insert(idx);
        }
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
