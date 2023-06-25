use emath::*;

use super::{OnCanvas, OnGrid, OnScreen};

impl OnScreen<Rect> {
    #[inline(always)]
    pub fn to_canvas(self, pixels_per_point: f32, zoom: f32) -> OnCanvas<Rect> {
        OnCanvas(Rect::from_min_max(
            OnScreen(self.0.min).to_canvas(pixels_per_point, zoom).0,
            OnScreen(self.0.max).to_canvas(pixels_per_point, zoom).0,
        ))
    }

    #[inline(always)]
    pub fn to_grid(self, pixels_per_point: f32, zoom: f32, tile_size: f32) -> OnGrid<Rect> {
        OnGrid(Rect::from_min_max(
            OnScreen(self.0.min).to_grid(pixels_per_point, zoom, tile_size).0,
            OnScreen(self.0.max).to_grid(pixels_per_point, zoom, tile_size).0,
        ))
    }
}

impl OnCanvas<Rect> {
    #[inline(always)]
    pub fn to_screen(self, pixels_per_point: f32, zoom: f32) -> OnScreen<Rect> {
        OnScreen(Rect::from_min_max(
            OnCanvas(self.0.min).to_screen(pixels_per_point, zoom).0,
            OnCanvas(self.0.max).to_screen(pixels_per_point, zoom).0,
        ))
    }

    #[inline(always)]
    pub fn to_grid(self, tile_size: f32) -> OnGrid<Rect> {
        OnGrid(Rect::from_min_max(
            //
            OnCanvas(self.0.min).to_grid(tile_size).0,
            OnCanvas(self.0.max).to_grid(tile_size).0,
        ))
    }
}

impl OnGrid<Rect> {
    #[inline(always)]
    pub fn to_screen(self, pixels_per_point: f32, zoom: f32, tile_size: f32) -> OnScreen<Rect> {
        OnScreen(Rect::from_min_max(
            OnGrid(self.0.min).to_screen(pixels_per_point, zoom, tile_size).0,
            OnGrid(self.0.max).to_screen(pixels_per_point, zoom, tile_size).0,
        ))
    }

    #[inline(always)]
    pub fn to_canvas(self, tile_size: f32) -> OnCanvas<Rect> {
        OnCanvas(Rect::from_min_max(
            OnGrid(self.0.min).to_canvas(tile_size).0,
            OnGrid(self.0.max).to_canvas(tile_size).0,
        ))
    }
}
