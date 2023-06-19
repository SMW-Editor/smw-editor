use std::ops::RangeInclusive;

use duplicate::duplicate_item;
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

#[duplicate_item(wrapper; [OnScreen]; [OnCanvas]; [OnGrid])]
impl wrapper<Rect> {
    #[inline(always)]
    pub const fn from_min_max(min: wrapper<Pos2>, max: wrapper<Pos2>) -> Self {
        Self(Rect::from_min_max(min.0, max.0))
    }

    #[inline(always)]
    pub fn from_min_size(min: wrapper<Pos2>, size: wrapper<Vec2>) -> Self {
        Self(Rect::from_min_size(min.0, size.0))
    }

    #[inline(always)]
    pub fn from_center_size(center: wrapper<Pos2>, size: wrapper<Vec2>) -> Self {
        Self(Rect::from_center_size(center.0, size.0))
    }

    #[inline(always)]
    pub fn from_x_y_ranges(x_range: impl Into<RangeInclusive<f32>>, y_range: impl Into<RangeInclusive<f32>>) -> Self {
        Self(Rect::from_x_y_ranges(x_range, y_range))
    }

    #[inline(always)]
    pub fn from_two_pos(a: wrapper<Pos2>, b: wrapper<Pos2>) -> Self {
        Self(Rect::from_two_pos(a.0, b.0))
    }

    pub fn from_points(points: &[wrapper<Pos2>]) -> Self {
        // Can't call Rect::from_points using the slice of wrapped Pos2 without an extra allocation.
        let mut rect = Rect::NOTHING;
        for p in points {
            rect.extend_with(p.0);
        }
        Self(rect)
    }

    #[inline]
    pub fn everything_right_of(left_x: f32) -> Self {
        Self(Rect::everything_right_of(left_x))
    }

    #[inline]
    pub fn everything_left_of(right_x: f32) -> Self {
        Self(Rect::everything_left_of(right_x))
    }

    #[inline]
    pub fn everything_below(top_y: f32) -> Self {
        Self(Rect::everything_below(top_y))
    }

    #[inline]
    pub fn everything_above(bottom_y: f32) -> Self {
        Self(Rect::everything_above(bottom_y))
    }

    #[must_use]
    pub fn expand(self, amnt: f32) -> Self {
        Self(self.0.expand(amnt))
    }

    #[must_use]
    pub fn expand2(self, amnt: Vec2) -> Self {
        Self(self.0.expand2(amnt))
    }

    #[must_use]
    pub fn shrink(self, amnt: f32) -> Self {
        Self(self.0.shrink(amnt))
    }

    #[must_use]
    pub fn shrink2(self, amnt: Vec2) -> Self {
        Self(self.0.shrink2(amnt))
    }

    #[inline]
    #[must_use]
    pub fn translate(self, amnt: Vec2) -> Self {
        Self(self.0.translate(amnt))
    }

    #[inline]
    #[must_use]
    pub fn rotate_bb(self, rot: Rot2) -> Self {
        Self(self.0.rotate_bb(rot))
    }

    #[inline]
    #[must_use]
    pub fn intersects(self, other: Self) -> bool {
        self.0.intersects(other.0)
    }

    #[inline(always)]
    pub fn set_width(&mut self, w: f32) {
        self.0.set_width(w)
    }

    #[inline(always)]
    pub fn set_height(&mut self, h: f32) {
        self.0.set_height(h)
    }

    #[inline(always)]
    pub fn set_center(&mut self, center: wrapper<Pos2>) {
        self.0.set_center(center.0)
    }

    #[inline]
    #[must_use]
    pub fn contains(self, other: wrapper<Pos2>) -> bool {
        self.0.contains(other.0)
    }

    #[inline]
    #[must_use]
    pub fn contains_rect(self, other: Self) -> bool {
        self.0.contains_rect(other.0)
    }

    #[must_use]
    pub fn clamp(self, p: wrapper<Pos2>) -> wrapper<Pos2> {
        wrapper(self.0.clamp(p.0))
    }

    #[inline(always)]
    pub fn extend_with(&mut self, p: wrapper<Pos2>) {
        self.0.extend_with(p.0)
    }

    #[inline(always)]
    pub fn extend_with_x(&mut self, x: f32) {
        self.0.extend_with_x(x)
    }

    #[inline(always)]
    pub fn extend_with_y(&mut self, y: f32) {
        self.0.extend_with_y(y)
    }

    #[inline(always)]
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        Self(self.0.union(other.0))
    }

    #[inline(always)]
    #[must_use]
    pub fn intersect(self, other: Self) -> Self {
        Self(self.0.intersect(other.0))
    }

    #[inline(always)]
    pub fn center(self) -> wrapper<Pos2> {
        wrapper(self.0.center())
    }

    #[inline(always)]
    pub fn size(self) -> wrapper<Vec2> {
        wrapper(self.0.size())
    }

    #[inline(always)]
    pub fn width(self) -> f32 {
        self.0.width()
    }

    #[inline(always)]
    pub fn height(self) -> f32 {
        self.0.height()
    }

    #[inline(always)]
    pub fn aspect_ratio(self) -> f32 {
        self.0.aspect_ratio()
    }

    #[inline(always)]
    pub fn square_proportions(self) -> wrapper<Vec2> {
        wrapper(self.0.square_proportions())
    }

    #[inline(always)]
    pub fn area(self) -> f32 {
        self.0.area()
    }

    #[inline]
    pub fn distance_to_pos(self, p: wrapper<Pos2>) -> f32 {
        self.0.distance_to_pos(p.0)
    }

    #[inline]
    pub fn distance_sq_to_pos(self, p: wrapper<Pos2>) -> f32 {
        self.0.distance_sq_to_pos(p.0)
    }

    #[inline]
    pub fn signed_distance_to_pos(self, p: wrapper<Pos2>) -> f32 {
        self.0.signed_distance_to_pos(p.0)
    }

    pub fn lerp_inside(self, t: wrapper<Vec2>) -> wrapper<Pos2> {
        wrapper(self.0.lerp_inside(t.0))
    }

    pub fn lerp_towards(self, other: &Self, t: f32) -> Self {
        Self(self.0.lerp_towards(&other.0, t))
    }

    #[inline(always)]
    pub fn x_range(self) -> RangeInclusive<f32> {
        self.0.x_range()
    }

    #[inline(always)]
    pub fn y_range(self) -> RangeInclusive<f32> {
        self.0.y_range()
    }

    #[inline(always)]
    pub fn bottom_up_range(self) -> RangeInclusive<f32> {
        self.0.bottom_up_range()
    }

    #[inline(always)]
    pub fn is_negative(self) -> bool {
        self.0.is_negative()
    }

    #[inline(always)]
    pub fn is_positive(self) -> bool {
        self.0.is_positive()
    }

    #[inline(always)]
    pub fn is_finite(self) -> bool {
        self.0.is_finite()
    }

    #[inline(always)]
    pub fn any_nan(self) -> bool {
        self.0.any_nan()
    }

    #[inline(always)]
    pub fn left(&self) -> f32 {
        self.0.left()
    }

    #[inline(always)]
    pub fn left_mut(&mut self) -> &mut f32 {
        self.0.left_mut()
    }

    #[inline(always)]
    pub fn set_left(&mut self, x: f32) {
        self.0.set_left(x)
    }

    #[inline(always)]
    pub fn right(&self) -> f32 {
        self.0.right()
    }

    #[inline(always)]
    pub fn right_mut(&mut self) -> &mut f32 {
        self.0.right_mut()
    }

    #[inline(always)]
    pub fn set_right(&mut self, x: f32) {
        self.0.set_right(x)
    }

    #[inline(always)]
    pub fn top(&self) -> f32 {
        self.0.top()
    }

    #[inline(always)]
    pub fn top_mut(&mut self) -> &mut f32 {
        self.0.top_mut()
    }

    #[inline(always)]
    pub fn set_top(&mut self, y: f32) {
        self.0.set_top(y)
    }

    #[inline(always)]
    pub fn bottom(&self) -> f32 {
        self.0.bottom()
    }

    #[inline(always)]
    pub fn bottom_mut(&mut self) -> &mut f32 {
        self.0.bottom_mut()
    }

    #[inline(always)]
    pub fn set_bottom(&mut self, y: f32) {
        self.0.set_bottom(y);
    }

    #[inline(always)]
    pub fn left_top(&self) -> wrapper<Pos2> {
        wrapper(self.0.left_top())
    }

    #[inline(always)]
    pub fn center_top(&self) -> wrapper<Pos2> {
        wrapper(self.0.center_top())
    }

    #[inline(always)]
    pub fn right_top(&self) -> wrapper<Pos2> {
        wrapper(self.0.right_top())
    }

    #[inline(always)]
    pub fn left_center(&self) -> wrapper<Pos2> {
        wrapper(self.0.left_center())
    }

    #[inline(always)]
    pub fn right_center(&self) -> wrapper<Pos2> {
        wrapper(self.0.right_center())
    }

    #[inline(always)]
    pub fn left_bottom(&self) -> wrapper<Pos2> {
        wrapper(self.0.left_bottom())
    }

    #[inline(always)]
    pub fn center_bottom(&self) -> wrapper<Pos2> {
        wrapper(self.0.center_bottom())
    }

    #[inline(always)]
    pub fn right_bottom(&self) -> wrapper<Pos2> {
        wrapper(self.0.right_bottom())
    }

    pub fn split_left_right_at_fraction(&self, t: f32) -> (Self, Self) {
        let (a, b) = self.0.split_left_right_at_fraction(t);
        (Self(a), Self(b))
    }

    pub fn split_left_right_at_x(&self, split_x: f32) -> (Self, Self) {
        let (a, b) = self.0.split_left_right_at_x(split_x);
        (Self(a), Self(b))
    }

    pub fn split_top_bottom_at_fraction(&self, t: f32) -> (Self, Self) {
        let (a, b) = self.0.split_top_bottom_at_fraction(t);
        (Self(a), Self(b))
    }

    pub fn split_top_bottom_at_y(&self, split_y: f32) -> (Self, Self) {
        let (a, b) = self.0.split_top_bottom_at_y(split_y);
        (Self(a), Self(b))
    }
}
