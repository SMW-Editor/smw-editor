use std::ops::{Div, Mul, MulAssign, Neg};

use duplicate::duplicate_item;
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

#[duplicate_item(wrapper; [OnScreen]; [OnCanvas]; [OnGrid])]
impl wrapper<Vec2> {
    #[inline(always)]
    pub fn new(x: f32, y: f32) -> Self {
        Self(vec2(x, y))
    }

    #[inline(always)]
    pub fn splat(v: f32) -> Self {
        Self(Vec2::splat(v))
    }

    #[inline(always)]
    pub fn to_pos2(self) -> wrapper<Pos2> {
        wrapper(self.0.to_pos2())
    }

    #[inline(always)]
    pub fn relative_to(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }

    #[inline(always)]
    #[must_use]
    pub fn normalized(self) -> Self {
        Self(self.0.normalized())
    }

    #[inline(always)]
    pub fn rot90(self) -> Self {
        Self(self.0.rot90())
    }

    #[inline(always)]
    pub fn length(self) -> f32 {
        self.0.length()
    }

    #[inline(always)]
    pub fn length_sq(self) -> f32 {
        self.0.length_sq()
    }

    #[inline(always)]
    pub fn angle(self) -> f32 {
        self.0.angle()
    }

    #[inline(always)]
    pub fn angled(angle: f32) -> Self {
        Self(Vec2::angled(angle))
    }

    #[inline]
    pub fn dot(self, other: Self) -> f32 {
        self.0.dot(other.0)
    }

    #[inline(always)]
    #[must_use]
    pub fn min_elem(self) -> f32 {
        self.0.min_elem()
    }

    #[inline(always)]
    #[must_use]
    pub fn max_elem(self) -> f32 {
        self.0.max_elem()
    }
}

#[duplicate_item(wrapper; [OnScreen]; [OnCanvas]; [OnGrid])]
impl Neg for wrapper<Vec2> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(self.0.neg())
    }
}

#[duplicate_item(wrapper; [OnScreen]; [OnCanvas]; [OnGrid])]
impl Mul for wrapper<Vec2> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0.mul(rhs.0))
    }
}

#[duplicate_item(wrapper; [OnScreen]; [OnCanvas]; [OnGrid])]
impl MulAssign<f32> for wrapper<Vec2> {
    fn mul_assign(&mut self, rhs: f32) {
        self.0.mul_assign(rhs);
    }
}

#[duplicate_item(wrapper; [OnScreen]; [OnCanvas]; [OnGrid])]
impl Mul<f32> for wrapper<Vec2> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0.mul(rhs))
    }
}

#[duplicate_item(wrapper; [OnScreen]; [OnCanvas]; [OnGrid])]
impl Mul<wrapper<Vec2>> for f32 {
    type Output = wrapper<Vec2>;

    fn mul(self, rhs: wrapper<Vec2>) -> Self::Output {
        wrapper(self.mul(rhs.0))
    }
}

#[duplicate_item(wrapper; [OnScreen]; [OnCanvas]; [OnGrid])]
impl Div for wrapper<Vec2> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0.div(rhs.0))
    }
}

#[duplicate_item(wrapper; [OnScreen]; [OnCanvas]; [OnGrid])]
impl Div<f32> for wrapper<Vec2> {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self(self.0.div(rhs))
    }
}
