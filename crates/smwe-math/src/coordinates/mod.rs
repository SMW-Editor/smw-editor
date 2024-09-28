pub mod pos2;
pub mod rect;
pub mod vec2;

use std::ops::*;

use shrinkwraprs::Shrinkwrap;
use wrapped_pos2_derive::WrappedPos2;
use wrapped_rect_derive::WrappedRect;
use wrapped_vec2_derive::WrappedVec2;

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

#[derive(
    Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Shrinkwrap, WrappedVec2, WrappedPos2, WrappedRect,
)]
#[shrinkwrap(mutable)]
pub struct OnScreen<T>(pub T);

#[derive(
    Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Shrinkwrap, WrappedVec2, WrappedPos2, WrappedRect,
)]
#[shrinkwrap(mutable)]
pub struct OnCanvas<T>(pub T);

#[derive(
    Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Shrinkwrap, WrappedVec2, WrappedPos2, WrappedRect,
)]
#[shrinkwrap(mutable)]
pub struct OnGrid<T>(pub T);
