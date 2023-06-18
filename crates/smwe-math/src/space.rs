/// # Term definitions
///
/// ## Screen
/// The area of your display; units are points.
///
/// ## Canvas
/// The editing area; units are canvas pixels.
///
/// ## Grid
/// The editing area divided into square cells; units are cell indices.
use emath::*;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct OnScreen<T>(pub T);

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct OnCanvas<T>(pub T);

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct OnGrid<T>(pub T);

impl OnScreen<Vec2> {
    #[inline]
    pub fn to_canvas(self, pixels_per_point: f32, zoom: f32) -> OnCanvas<Vec2> {
        let canvas = self.0 * pixels_per_point / zoom;
        OnCanvas(canvas.floor())
    }

    #[inline]
    pub fn to_grid(self, pixels_per_point: f32, zoom: f32, tile_size: f32) -> OnGrid<Vec2> {
        self.to_canvas(pixels_per_point, zoom).to_grid(tile_size)
    }

    #[inline]
    pub fn relative_to(self, other: Self) -> OnScreen<Vec2> {
        OnScreen(self.0 - other.0)
    }

    #[inline]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self(self.0.clamp(min.0, max.0))
    }

    #[inline]
    pub fn to_pos2(self) -> OnScreen<Pos2> {
        OnScreen(self.0.to_pos2())
    }
}

impl OnCanvas<Vec2> {
    #[inline]
    pub fn to_screen(self, pixels_per_point: f32, zoom: f32) -> OnScreen<Vec2> {
        let screen = self.0 * zoom / pixels_per_point;
        OnScreen(screen)
    }

    #[inline]
    pub fn to_grid(self, tile_size: f32) -> OnGrid<Vec2> {
        let grid = self.0 / tile_size;
        OnGrid(grid.floor())
    }

    #[inline]
    pub fn relative_to(self, other: Self) -> OnScreen<Vec2> {
        OnScreen(self.0 - other.0)
    }

    #[inline]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self(self.0.clamp(min.0, max.0))
    }

    #[inline]
    pub fn to_pos2(self) -> OnCanvas<Pos2> {
        OnCanvas(self.0.to_pos2())
    }
}

impl OnGrid<Vec2> {
    #[inline]
    pub fn to_screen(self, pixels_per_point: f32, zoom: f32, tile_size: f32) -> OnScreen<Vec2> {
        self.to_canvas(tile_size).to_screen(pixels_per_point, zoom)
    }

    #[inline]
    pub fn to_canvas(self, tile_size: f32) -> OnCanvas<Vec2> {
        let canvas = self.0 * tile_size;
        OnCanvas(canvas)
    }

    #[inline]
    pub fn relative_to(self, other: Self) -> OnScreen<Vec2> {
        OnScreen(self.0 - other.0)
    }

    #[inline]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self(self.0.clamp(min.0, max.0))
    }

    #[inline]
    pub fn to_pos2(self) -> OnGrid<Pos2> {
        OnGrid(self.0.to_pos2())
    }
}

impl OnScreen<Pos2> {
    #[inline]
    pub fn to_canvas(self, pixels_per_point: f32, zoom: f32) -> OnCanvas<Pos2> {
        self.to_vec2().to_canvas(pixels_per_point, zoom).to_pos2()
    }

    #[inline]
    pub fn to_grid(self, pixels_per_point: f32, zoom: f32, tile_size: f32) -> OnGrid<Pos2> {
        self.to_vec2().to_grid(pixels_per_point, zoom, tile_size).to_pos2()
    }

    #[inline]
    pub fn relative_to(self, other: Self) -> OnScreen<Pos2> {
        OnScreen(self.0 - other.0.to_vec2())
    }

    #[inline]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self(self.0.clamp(min.0, max.0))
    }

    #[inline]
    pub fn to_vec2(self) -> OnScreen<Vec2> {
        OnScreen(self.0.to_vec2())
    }
}

impl OnCanvas<Pos2> {
    #[inline]
    pub fn to_screen(self, pixels_per_point: f32, zoom: f32) -> OnScreen<Pos2> {
        self.to_vec2().to_screen(pixels_per_point, zoom).to_pos2()
    }

    #[inline]
    pub fn to_grid(self, tile_size: f32) -> OnGrid<Pos2> {
        self.to_vec2().to_grid(tile_size).to_pos2()
    }

    #[inline]
    pub fn relative_to(self, other: Self) -> OnScreen<Pos2> {
        OnScreen(self.0 - other.0.to_vec2())
    }

    #[inline]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self(self.0.clamp(min.0, max.0))
    }

    #[inline]
    pub fn to_vec2(self) -> OnCanvas<Vec2> {
        OnCanvas(self.0.to_vec2())
    }
}

impl OnGrid<Pos2> {
    #[inline]
    pub fn to_screen(self, pixels_per_point: f32, zoom: f32, tile_size: f32) -> OnScreen<Pos2> {
        self.to_vec2().to_screen(pixels_per_point, zoom, tile_size).to_pos2()
    }

    #[inline]
    pub fn to_canvas(self, tile_size: f32) -> OnCanvas<Pos2> {
        self.to_vec2().to_canvas(tile_size).to_pos2()
    }

    #[inline]
    pub fn relative_to(self, other: Self) -> OnScreen<Pos2> {
        OnScreen(self.0 - other.0.to_vec2())
    }

    #[inline]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self(self.0.clamp(min.0, max.0))
    }

    #[inline]
    pub fn to_vec2(self) -> OnGrid<Vec2> {
        OnGrid(self.0.to_vec2())
    }
}

impl OnScreen<Rect> {
    #[inline]
    pub fn to_canvas(self, pixels_per_point: f32, zoom: f32) -> OnCanvas<Rect> {
        OnCanvas(Rect::from_min_max(
            OnScreen(self.0.min).to_canvas(pixels_per_point, zoom).0,
            OnScreen(self.0.max).to_canvas(pixels_per_point, zoom).0,
        ))
    }

    #[inline]
    pub fn to_grid(self, pixels_per_point: f32, zoom: f32, tile_size: f32) -> OnGrid<Rect> {
        OnGrid(Rect::from_min_max(
            OnScreen(self.0.min).to_grid(pixels_per_point, zoom, tile_size).0,
            OnScreen(self.0.max).to_grid(pixels_per_point, zoom, tile_size).0,
        ))
    }
}

impl OnCanvas<Rect> {
    #[inline]
    pub fn to_screen(self, pixels_per_point: f32, zoom: f32) -> OnScreen<Rect> {
        OnScreen(Rect::from_min_max(
            OnCanvas(self.0.min).to_screen(pixels_per_point, zoom).0,
            OnCanvas(self.0.max).to_screen(pixels_per_point, zoom).0,
        ))
    }

    #[inline]
    pub fn to_grid(self, tile_size: f32) -> OnGrid<Rect> {
        OnGrid(Rect::from_min_max(
            //
            OnCanvas(self.0.min).to_grid(tile_size).0,
            OnCanvas(self.0.max).to_grid(tile_size).0,
        ))
    }
}

impl OnGrid<Rect> {
    #[inline]
    pub fn to_screen(self, pixels_per_point: f32, zoom: f32, tile_size: f32) -> OnScreen<Rect> {
        OnScreen(Rect::from_min_max(
            OnGrid(self.0.min).to_screen(pixels_per_point, zoom, tile_size).0,
            OnGrid(self.0.max).to_screen(pixels_per_point, zoom, tile_size).0,
        ))
    }

    #[inline]
    pub fn to_canvas(self, tile_size: f32) -> OnCanvas<Rect> {
        OnCanvas(Rect::from_min_max(
            OnGrid(self.0.min).to_canvas(tile_size).0,
            OnGrid(self.0.max).to_canvas(tile_size).0,
        ))
    }
}
