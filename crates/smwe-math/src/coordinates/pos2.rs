use std::ops::Sub;

use duplicate::duplicate_item;
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

#[duplicate_item(wrapper; [OnScreen]; [OnCanvas]; [OnGrid])]
impl wrapper<Pos2> {
    pub const ZERO: Self = Self::new(0., 0.);

    #[inline(always)]
    pub const fn new(x: f32, y: f32) -> Self {
        Self(pos2(x, y))
    }

    #[inline(always)]
    pub fn to_vec2(self) -> wrapper<Vec2> {
        wrapper(self.0.to_vec2())
    }

    #[inline(always)]
    pub fn relative_to(self, other: Self) -> Self {
        Self(self.0 - other.0.to_vec2())
    }

    #[inline]
    pub fn distance(self, other: Self) -> f32 {
        self.0.distance(other.0)
    }

    #[inline]
    pub fn distance_sq(self, other: Self) -> f32 {
        self.0.distance_sq(other.0)
    }

    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self(self.0.lerp(other.0, t))
    }
}

#[duplicate_item(wrapper; [OnScreen]; [OnCanvas]; [OnGrid])]
impl Sub for wrapper<Pos2> {
    type Output = wrapper<Vec2>;

    fn sub(self, rhs: Self) -> Self::Output {
        wrapper(self.0.sub(rhs.0))
    }
}
