use std::ops::Not;

use duplicate::duplicate;
use egui::{pos2, Rect};
use itertools::Itertools;
use paste::paste;
use smwe_math::coordinates::OnCanvas;

use super::super::UiSpriteMapEditor;

impl UiSpriteMapEditor {
    pub(in super::super) fn select_all_tiles(&mut self) {
        self.mark_tiles_as_selected(0..self.sprite_tiles.read(|tiles| tiles.len()));
    }

    pub(in super::super) fn unselect_all_tiles(&mut self) {
        self.selected_sprite_tile_indices.clear();
        self.selection_bounds = None;
    }

    pub(in super::super) fn mark_tiles_as_selected(&mut self, indices: impl IntoIterator<Item = usize>) {
        for index in indices {
            self.selected_sprite_tile_indices.insert(index);
        }
        self.compute_selection_bounds();
    }

    pub(in super::super) fn compute_selection_bounds(&mut self) {
        self.selection_bounds = self.selected_sprite_tile_indices.is_empty().not().then(|| {
            duplicate! {
                [dimension; [x]; [y]]
                paste! {
                    let ([<min_tile_ dimension>], [<max_tile_ dimension>]) = self.sprite_tiles.read(|tiles| {
                        self
                            .selected_sprite_tile_indices
                            .iter()
                            .map(|&i| tiles[i].pos())
                            .minmax_by(|a, b| a.dimension.total_cmp(&b.dimension))
                            .into_option()
                            .map(|(min, max)| (min.dimension, max.dimension))
                            .unwrap()
                    });
                }
            }
            OnCanvas(Rect::from_min_max(pos2(min_tile_x, min_tile_y), pos2(max_tile_x, max_tile_y)))
        });
    }
}
