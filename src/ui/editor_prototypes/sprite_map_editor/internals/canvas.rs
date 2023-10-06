use egui::{PlatformOutput, Pos2, Rangef, Rect, Vec2};
use itertools::Itertools;
use num::Integer;
use smwe_math::coordinates::{OnCanvas, OnGrid, OnScreen};
use smwe_render::tile_renderer::{Tile, TileJson};

use super::super::UiSpriteMapEditor;
use crate::ui::editing_mode::{FlipDirection, SnapToGrid};

impl UiSpriteMapEditor {
    pub(in super::super) fn canvas_size(&self) -> OnScreen<Vec2> {
        OnGrid::splat(32.).to_screen(self.pixels_per_point, self.zoom, self.tile_size_px)
    }

    pub(in super::super) fn any_selected_tile_contains_point(&self, point: OnCanvas<Pos2>) -> bool {
        self.sprite_tiles
            .read(|tiles| self.selected_sprite_tile_indices.iter().copied().any(|i| tiles[i].contains_point(point)))
    }

    pub(in super::super) fn find_tile_containing_point(&self, point: OnCanvas<Pos2>) -> Option<(usize, Tile)> {
        self.sprite_tiles.read(|tiles| tiles.iter().copied().enumerate().find(|(_, tile)| tile.contains_point(point)))
    }

    pub(in super::super) fn select_tile_at(&mut self, pos: OnScreen<Pos2>, clear_previous_selection: bool) {
        if clear_previous_selection {
            self.unselect_all_tiles();
        }

        if let Some((idx, _)) = self.find_tile_containing_point(pos.to_canvas(self.pixels_per_point, self.zoom)) {
            self.selected_sprite_tile_indices.insert(idx);
        }

        self.compute_selection_bounds();
    }

    pub(in super::super) fn select_tiles_inside(&mut self, rect: OnScreen<Rect>, clear_previous_selection: bool) {
        if clear_previous_selection {
            self.unselect_all_tiles();
        }

        let indices = self.sprite_tiles.read(|tiles| {
            tiles
                .iter()
                .enumerate()
                .filter_map(|(index, &tile)| {
                    tile.intersects_rect(rect.to_canvas(self.pixels_per_point, self.zoom)).then_some(index)
                })
                .collect_vec()
        });
        self.mark_tiles_as_selected(indices);
    }

    pub(in super::super) fn move_selected_tiles_by(
        &mut self, move_offset: OnCanvas<Vec2>, snap_to_grid: Option<SnapToGrid>,
    ) {
        if self.selected_sprite_tile_indices.is_empty() {
            return;
        }

        let bounds = self.selection_bounds.expect("unset even though some tiles are selected");
        let move_offset = move_offset.clamp(
            -bounds.left_top().to_vec2(),
            OnCanvas::splat(31. * self.tile_size_px) - bounds.right_bottom().to_vec2(),
        );

        self.sprite_tiles.write(|tiles| {
            for &idx in self.selected_sprite_tile_indices.iter() {
                tiles[idx].move_by(move_offset);
                if let Some(snap_to_grid) = snap_to_grid {
                    tiles[idx].snap_to_grid(self.tile_size_px as u32, snap_to_grid.cell_origin);
                }
            }
        });

        self.compute_selection_bounds();
        self.upload_tiles();
    }

    pub(in super::super) fn add_selected_tile_at(&mut self, pos: OnCanvas<Pos2>) {
        let tile_idx = (self.selected_vram_tile.0 + self.selected_vram_tile.1 * 16) as usize;
        let tile = self.tile_palette[tile_idx + (32 * 16)];
        self.add_tile_at(tile, pos);
    }

    pub(in super::super) fn add_tile_at(&mut self, mut tile: Tile, pos: OnCanvas<Pos2>) {
        tile.move_to(pos.floor());
        self.add_tiles([tile]);
    }

    pub(in super::super) fn add_tiles(&mut self, new_tiles: impl IntoIterator<Item = Tile>) {
        self.sprite_tiles.write(|tiles| {
            self.selected_sprite_tile_indices.insert(tiles.len());
            tiles.extend(new_tiles);
        });
    }

    pub(in super::super) fn copy_selected_tiles(&self, platform_output: &mut PlatformOutput) {
        let selected_tiles = self
            .selected_sprite_tile_indices
            .iter()
            .map(|&i| self.sprite_tiles.read(|tiles| tiles[i]))
            .map(|mut t| {
                let bounds = self.selection_bounds.expect("No selection bounds even though tiles are selected");
                t.move_by(-bounds.left_top().to_vec2());
                t
            })
            .map(TileJson::from)
            .collect_vec();
        platform_output.copied_text =
            serde_json::to_string(&selected_tiles).expect("Failed to serialize selected tiles");
    }

    pub(in super::super) fn paste_tiles_to(&mut self, tiles: Vec<TileJson>, paste_offset: OnCanvas<Vec2>) {
        self.unselect_all_tiles();

        let tiles = tiles
            .into_iter()
            .map(Tile::from)
            .map(|mut tile| {
                tile.move_by(paste_offset);
                tile
            })
            .collect_vec();
        self.add_tiles(tiles);

        self.compute_selection_bounds();
        self.upload_tiles();
    }

    pub(in super::super) fn delete_selected_tiles(&mut self) {
        self.sprite_tiles.write(|tiles| {
            for idx in self.selected_sprite_tile_indices.drain().sorted().rev() {
                tiles.remove(idx);
            }
        });
        self.selection_bounds = None;
        self.upload_tiles();
    }

    pub(in super::super) fn delete_tiles_at(&mut self, pos: OnScreen<Pos2>) {
        self.sprite_tiles
            .write(|tiles| tiles.retain(|&tile| !tile.contains_point(pos.to_canvas(self.pixels_per_point, self.zoom))));
        self.upload_tiles();
    }

    pub(in super::super) fn probe_tile_at(&mut self, pos: OnScreen<Pos2>) {
        self.sprite_tiles.read(|tiles| {
            if let Some(tile) =
                tiles.iter().rev().find(|&&tile| tile.contains_point(pos.to_canvas(self.pixels_per_point, self.zoom)))
            {
                let (y, x) = tile.tile_num().div_rem(&16);
                self.selected_vram_tile = (x, y - 96);
            }
        });
    }

    pub(in super::super) fn flip_selected_tiles(&mut self, flip_direction: FlipDirection) {
        let selection_bounds = self.selection_bounds.expect("unset even though some tiles are selected");
        let Rangef { min: x_min, max: x_max } = selection_bounds.x_range();
        let Rangef { min: y_min, max: y_max } = selection_bounds.y_range();
        self.sprite_tiles.write(|tiles| {
            for &i in self.selected_sprite_tile_indices.iter() {
                let tile = &mut tiles[i];
                match flip_direction {
                    FlipDirection::Horizontal => {
                        tile.toggle_flip_x();
                        tile[0] = (x_min + (x_max - tile[0] as f32)) as u32;
                    }
                    FlipDirection::Vertical => {
                        tile.toggle_flip_y();
                        tile[1] = (y_min + (y_max - tile[1] as f32)) as u32;
                    }
                }
            }
        });
        self.upload_tiles();
    }
}
