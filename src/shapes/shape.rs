#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::LinePixelIterator;
use crate::{CirclePixelIterator, ICircle, ILine, RectPixelIterator};
use bevy_math::{IRect, IVec2};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Shape {
    Point { point: IVec2 },
    Line { line: ILine },
    Circle { circle: ICircle },
    Rectangle { rect: IRect },
}

impl Shape {
    #[inline]
    #[must_use]
    pub fn aabb(&self) -> IRect {
        match self {
            Shape::Point { point } => IRect::from_corners(*point, *point),
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
                iter: circle.pixels(),
            },
            Shape::Rectangle { rect } => ShapePixelIterator::Rectangle {
                iter: RectPixelIterator::new(*rect),
            },
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ShapePixelIterator {
    Point { iter: PointPixelIterator },
    Line { iter: LinePixelIterator },
    Circle { iter: CirclePixelIterator },
    Rectangle { iter: RectPixelIterator },
}

impl Iterator for ShapePixelIterator {
    type Item = IVec2;

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
    point: Option<IVec2>,
}

impl Iterator for PointPixelIterator {
    type Item = IVec2;

    fn next(&mut self) -> Option<Self::Item> {
        self.point.take()
    }
}
