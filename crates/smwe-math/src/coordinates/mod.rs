pub mod pos2;
pub mod rect;
pub mod vec2;

use std::ops::{Add, AddAssign, Index, IndexMut, Sub, SubAssign};

/// # Coordinate systems
///
/// ## Screen
/// The area of your display; units are points.
///
/// ## Canvas
/// The editing area; units are canvas pixels.
///
/// ## Grid
/// The editing area divided into square cells; units are cell indices.
use duplicate::{duplicate, duplicate_item};
use emath::*;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct OnScreen<T>(pub T);

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct OnCanvas<T>(pub T);

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct OnGrid<T>(pub T);

#[duplicate_item(
    wrapper;
    duplicate! {
        [inner; [Vec2]; [Pos2]]
        [OnScreen<inner>];
        [OnCanvas<inner>];
        [OnGrid<inner>];
    }
)]
impl wrapper {
    #[inline(always)]
    pub fn floor(self) -> Self {
        Self(self.0.floor())
    }

    #[inline(always)]
    pub fn round(self) -> Self {
        Self(self.0.round())
    }

    #[inline(always)]
    pub fn ceil(self) -> Self {
        Self(self.0.ceil())
    }

    #[inline(always)]
    pub fn is_finite(self) -> bool {
        self.0.is_finite()
    }

    #[inline(always)]
    pub fn any_nan(self) -> bool {
        self.0.any_nan()
    }

    #[inline]
    #[must_use]
    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    #[inline]
    #[must_use]
    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }

    #[inline]
    #[must_use]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self(self.0.clamp(min.0, max.0))
    }
}

#[duplicate_item(
    wrapper;
    duplicate! {
        [inner; [Vec2]; [Pos2]]
        [OnScreen<inner>];
        [OnCanvas<inner>];
        [OnGrid<inner>];
    }
)]
impl Index<usize> for wrapper {
    type Output = f32;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

#[duplicate_item(
    wrapper;
    duplicate! {
        [inner; [Vec2]; [Pos2]]
        [OnScreen<inner>];
        [OnCanvas<inner>];
        [OnGrid<inner>];
    }
)]
impl IndexMut<usize> for wrapper {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

duplicate! {
    [inner; [Vec2]; [Pos2]]
    #[duplicate_item(
        wrapper;
        [OnScreen];
        [OnCanvas];
        [OnGrid];
    )]
    impl AddAssign<wrapper<Vec2>> for wrapper<inner> {
        fn add_assign(&mut self, rhs: wrapper<Vec2>) {
            self.0.add_assign(rhs.0);
        }
    }
}

duplicate! {
    [inner; [Vec2]; [Pos2]]
    #[duplicate_item(
        wrapper;
        [OnScreen];
        [OnCanvas];
        [OnGrid];
    )]
    impl SubAssign<wrapper<Vec2>> for wrapper<inner> {
        fn sub_assign(&mut self, rhs: wrapper<Vec2>) {
            self.0.sub_assign(rhs.0);
        }
    }
}

duplicate! {
    [inner; [Vec2]; [Pos2]]
    #[duplicate_item(
        wrapper;
        [OnScreen];
        [OnCanvas];
        [OnGrid];
    )]
    impl Add<wrapper<Vec2>> for wrapper<inner> {
        type Output = Self;

        fn add(self, rhs: wrapper<Vec2>) -> Self::Output {
            Self(self.0.add(rhs.0))
        }
    }
}

duplicate! {
    [inner; [Vec2]; [Pos2]]
    #[duplicate_item(
        wrapper;
        [OnScreen];
        [OnCanvas];
        [OnGrid];
    )]
    impl Sub<wrapper<Vec2>> for wrapper<inner> {
        type Output = Self;

        fn sub(self, rhs: wrapper<Vec2>) -> Self::Output {
            Self(self.0.sub(rhs.0))
        }
    }
}
