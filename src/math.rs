use crate::{iline, ILine};
use bevy_math::{ivec2, uvec2, vec2, IRect, IVec2, Rect, URect, UVec2, Vec2};

/// Find the distance squared between two points.
#[inline]
#[must_use]
pub fn distance_squared_to_upoint(a: UVec2, b: UVec2) -> f32 {
    a.as_vec2().distance_squared(b.as_vec2())
}

/// Find the distance between two points.
#[inline]
#[must_use]
pub fn distance_to_upoint(a: UVec2, b: UVec2) -> f32 {
    distance_squared_to_upoint(a, b).sqrt()
}

/// Find the distance squared between two points.
#[inline]
#[must_use]
pub fn distance_squared_to_ipoint(a: IVec2, b: IVec2) -> f32 {
    a.as_vec2().distance_squared(b.as_vec2())
}

/// Find the distance between two points.
#[inline]
#[must_use]
pub fn distance_to_ipoint(a: IVec2, b: IVec2) -> f32 {
    distance_squared_to_ipoint(a, b).sqrt()
}

/// Get the four points that make up the corners of the given `rect`.
#[inline]
#[must_use]
pub fn rect_points(rect: &Rect) -> [Vec2; 4] {
    [
        rect.min,
        rect.min + vec2(rect.width(), 0.),
        rect.max,
        rect.min + vec2(0., rect.height()),
    ]
}

/// Get the four points that make up the corners of the given `rect`.
#[inline]
#[must_use]
pub fn irect_points(rect: &IRect) -> [IVec2; 4] {
    [
        rect.min,
        rect.min + ivec2(rect.width(), 0),
        rect.max,
        rect.min + ivec2(0, rect.height()),
    ]
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
pub fn irect_edges(rect: &IRect) -> [ILine; 4] {
    let min = rect.min;
    let max = rect.max;
    let width = rect.width();
    let height = rect.height();
    [
        iline(min, min + ivec2(width, 0)),
        iline(min + ivec2(width, 0), max),
        iline(max, min + ivec2(0, height)),
        iline(min + ivec2(0, height), min),
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
pub fn exclusive_irect(rect: &IRect) -> IRect {
    if rect.is_empty() {
        return *rect;
    }
    let max = rect.max - IVec2::ONE;
    IRect::from_corners(rect.min, max.max(rect.min))
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
