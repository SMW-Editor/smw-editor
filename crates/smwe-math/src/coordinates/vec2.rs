use emath::*;

use super::{OnCanvas, OnGrid, OnScreen};

impl OnScreen<Vec2> {
    #[inline(always)]
    pub fn to_canvas(self, pixels_per_point: f32, zoom: f32) -> OnCanvas<Vec2> {
        OnCanvas(self.0 * pixels_per_point / zoom).floor()
    }

    #[inline(always)]
    pub fn to_grid(self, pixels_per_point: f32, zoom: f32, tile_size: f32) -> OnGrid<Vec2> {
        self.to_canvas(pixels_per_point, zoom).to_grid(tile_size)
    }
}

impl OnCanvas<Vec2> {
    #[inline(always)]
    pub fn to_screen(self, pixels_per_point: f32, zoom: f32) -> OnScreen<Vec2> {
        OnScreen(self.0 * zoom / pixels_per_point)
    }

    #[inline(always)]
    pub fn to_grid(self, tile_size: f32) -> OnGrid<Vec2> {
        OnGrid(self.0 / tile_size).floor()
    }
}

impl OnGrid<Vec2> {
    #[inline(always)]
    pub fn to_screen(self, pixels_per_point: f32, zoom: f32, tile_size: f32) -> OnScreen<Vec2> {
        self.to_canvas(tile_size).to_screen(pixels_per_point, zoom)
    }

    #[inline(always)]
    pub fn to_canvas(self, tile_size: f32) -> OnCanvas<Vec2> {
        OnCanvas(self.0 * tile_size)
    }
}
