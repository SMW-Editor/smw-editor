use emath::*;

use super::{OnCanvas, OnGrid, OnScreen};

impl OnScreen<Pos2> {
    #[inline(always)]
    pub fn to_canvas(self, pixels_per_point: f32, zoom: f32) -> OnCanvas<Pos2> {
        self.to_vec2().to_canvas(pixels_per_point, zoom).to_pos2()
    }

    #[inline(always)]
    pub fn to_grid(self, pixels_per_point: f32, zoom: f32, tile_size: f32) -> OnGrid<Pos2> {
        self.to_vec2().to_grid(pixels_per_point, zoom, tile_size).to_pos2()
    }
}

impl OnCanvas<Pos2> {
    #[inline(always)]
    pub fn to_screen(self, pixels_per_point: f32, zoom: f32) -> OnScreen<Pos2> {
        self.to_vec2().to_screen(pixels_per_point, zoom).to_pos2()
    }

    #[inline(always)]
    pub fn to_grid(self, tile_size: f32) -> OnGrid<Pos2> {
        self.to_vec2().to_grid(tile_size).to_pos2()
    }
}

impl OnGrid<Pos2> {
    #[inline(always)]
    pub fn to_screen(self, pixels_per_point: f32, zoom: f32, tile_size: f32) -> OnScreen<Pos2> {
        self.to_vec2().to_screen(pixels_per_point, zoom, tile_size).to_pos2()
    }

    #[inline(always)]
    pub fn to_canvas(self, tile_size: f32) -> OnCanvas<Pos2> {
        self.to_vec2().to_canvas(tile_size).to_pos2()
    }
}
