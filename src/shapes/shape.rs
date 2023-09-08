#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{CroppedCirclePixelIterator, LinePixelIterator};
use crate::{ICircle, ULine, URectPixelIterator};
use bevy_math::{URect, UVec2};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Shape {
    Point { point: UVec2 },
    Line { line: ULine },
    Circle { circle: ICircle },
    Rectangle { rect: URect },
}

impl Shape {
    #[inline]
    #[must_use]
    pub fn aabb(&self) -> URect {
        match self {
            Shape::Point { point } => URect::from_corners(*point, *point),
            Shape::Line { line } => line.aabb(),
            Shape::Circle { circle } => circle.aabb(),
            Shape::Rectangle { rect } => *rect,
        }
    }

    #[inline]
    #[must_use]
    pub fn pixels(&self) -> ShapePixelIterator {
        match self {
            Shape::Point { point } => ShapePixelIterator::Point {
                iter: PointPixelIterator {
                    point: Some(*point),
                },
            },
            Shape::Line { line } => ShapePixelIterator::Line {
                iter: line.pixels(),
            },
            Shape::Circle { circle } => ShapePixelIterator::Circle {
                iter: circle.cropped_pixels(),
            },
            Shape::Rectangle { rect } => ShapePixelIterator::Rectangle {
                iter: URectPixelIterator::new(*rect),
            },
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ShapePixelIterator {
    Point { iter: PointPixelIterator },
    Line { iter: LinePixelIterator },
    Circle { iter: CroppedCirclePixelIterator },
    Rectangle { iter: URectPixelIterator },
}

impl Iterator for ShapePixelIterator {
    type Item = UVec2;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ShapePixelIterator::Point { iter } => iter.next(),
            ShapePixelIterator::Line { iter } => iter.next(),
            ShapePixelIterator::Circle { iter } => iter.next(),
            ShapePixelIterator::Rectangle { iter } => iter.next(),
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PointPixelIterator {
    point: Option<UVec2>,
}

impl Iterator for PointPixelIterator {
    type Item = UVec2;

    fn next(&mut self) -> Option<Self::Item> {
        self.point.take()
    }
}
