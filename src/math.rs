use super::ULine;
use bevy_math::{uvec2, IRect, IVec2, URect, UVec2};

/// Find the distance squared between two points.
#[inline]
#[must_use]
pub fn distance_squared_to(a: UVec2, b: UVec2) -> f32 {
    a.as_vec2().distance_squared(b.as_vec2())
}

/// Find the distance between two points.
#[inline]
#[must_use]
pub fn distance_to(a: UVec2, b: UVec2) -> f32 {
    distance_squared_to(a, b).sqrt()
}

/// Get the four points that make up the corners of the given `rect`.
#[inline]
#[must_use]
pub fn urect_points(rect: &URect) -> [UVec2; 4] {
    [
        rect.min,
        rect.min + uvec2(rect.width(), 0),
        rect.max,
        rect.min + uvec2(0, rect.height()),
    ]
}

/// Get the four lines that make up the edges of this rectangle.
#[inline]
#[must_use]
pub fn urect_edges(rect: &URect) -> [ULine; 4] {
    let width = rect.width();
    let height = rect.height();
    [
        ULine::new(rect.min, rect.min + uvec2(width, 0)),
        ULine::new(rect.min + uvec2(width, 0), rect.max),
        ULine::new(rect.max, rect.min + uvec2(0, height)),
        ULine::new(rect.min + uvec2(0, height), rect.min),
    ]
}

/// Convert the given `IRect` into a `URect`, without wrapping negative values,
/// effectively cropping the rectangle to the positive quadrant.
#[inline]
#[must_use]
pub fn to_cropped_urect(rect: &IRect) -> URect {
    URect::from_corners(
        rect.min.max(IVec2::ZERO).as_uvec2(),
        rect.max.max(IVec2::ZERO).as_uvec2(),
    )
}

/// Subtract one from the maximum point of the given `rect`, allowing
/// for exclusive handling with `contains`, for example.
#[inline]
#[must_use]
pub fn exclusive_urect(rect: &URect) -> URect {
    if rect.is_empty() || rect.max.x == 0 || rect.max.y == 0 {
        return *rect;
    }
    let max = rect.max - UVec2::ONE;
    URect::from_corners(rect.min, max.max(rect.min))
}
