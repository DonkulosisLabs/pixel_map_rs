#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::ILine;
use crate::{Direction, Region};
use glam::IVec2;

use num_traits::{NumCast, Unsigned};
use std::ops;

/// An immutable rectangle defined by a minimum and maximum point, in integer coordinates.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct IRect {
    min: IVec2,
    max: IVec2,
}

impl IRect {
    pub const ZERO: Self = Self {
        min: IVec2::ZERO,
        max: IVec2::ZERO,
    };

    /// Create a new [IRect] with the given minimum and maximum points.
    #[inline]
    #[must_use]
    pub fn new(x0: i32, y0: i32, x1: i32, y1: i32) -> Self {
        Self::from_corners((x0, y0), (x1, y1))
    }

    /// Create a new [IRect] with the given minimum and maximum points.
    /// Points are normalized such that the minimum becomes the bottom left corner,
    /// and the maximum becomes the top right corner.
    #[inline]
    #[must_use]
    pub fn from_corners<P>(min: P, max: P) -> Self
    where
        P: Into<IVec2>,
    {
        let min = min.into();
        let max = max.into();
        Self {
            min: min.min(max),
            max: min.max(max),
        }
    }

    /// Create a new [IRect] with the given center point, and width and height.
    #[inline]
    #[must_use]
    pub fn centered_at<P>(point: P, width: u32, height: u32) -> Self
    where
        P: Into<IVec2>,
    {
        let point = point.into();
        if width <= 1 || height <= 1 {
            return Self {
                min: point,
                max: point + IVec2::new(width as i32, height as i32),
            };
        }

        let width_half = width as i32 / 2;
        let height_half = height as i32 / 2;
        let min = point - IVec2::new(width_half, height_half);
        let max = point + IVec2::new(width_half, height_half);

        Self::from_corners(min, max)
    }

    /// Determine if this rectangle min point is equal to the max point.
    #[inline]
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.min.x == self.max.x && self.min.y == self.max.y
    }

    /// Determine if this rectangle's area is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.min.x == self.max.x || self.min.y == self.max.y
    }

    /// Get the minimum point `x` component.
    #[inline]
    #[must_use]
    pub fn x(&self) -> i32 {
        self.min.x
    }

    /// Get the minimum point `y` component.
    #[inline]
    #[must_use]
    pub fn y(&self) -> i32 {
        self.min.y
    }

    /// Get the minimum point.
    #[inline]
    #[must_use]
    pub fn min(&self) -> IVec2 {
        self.min
    }

    /// Get the maximum point.
    #[inline]
    #[must_use]
    pub fn max(&self) -> IVec2 {
        self.max
    }

    /// Get the center point of this rectangle.
    #[inline]
    #[must_use]
    pub fn center(&self) -> IVec2 {
        let half_max = self.max / 2;
        self.min + half_max
    }

    /// Get the width of this rectangle.
    #[inline]
    #[must_use]
    pub fn width(&self) -> u32 {
        (self.max.x - self.min.x) as u32
    }

    /// Get the height of this rectangle.
    #[inline]
    #[must_use]
    pub fn height(&self) -> u32 {
        (self.max.y - self.min.y) as u32
    }

    /// Get the dimensions of this rectangle.
    #[inline]
    #[must_use]
    pub fn size(&self) -> IVec2 {
        self.max - self.min
    }

    /// Get the area of this rectangle.
    /// This is equivalent to `width * height`.
    #[inline]
    #[must_use]
    pub fn area(&self) -> u32 {
        self.width() * self.height()
    }

    /// Get the left bounds of this rectangle, inclusive.
    #[inline]
    #[must_use]
    pub fn left_bounds(&self) -> i32 {
        self.min.x
    }

    /// Get the right bounds of this rectangle, exclusive.
    #[inline]
    #[must_use]
    pub fn right_bounds(&self) -> i32 {
        self.max.x - 1
    }

    /// Get the top bounds of this rectangle, exclusive.
    #[inline]
    #[must_use]
    pub fn top_bounds(&self) -> i32 {
        self.max.y - 1
    }

    /// Get the bottom bounds of this rectangle, inclusive.
    #[inline]
    #[must_use]
    pub fn bottom_bounds(&self) -> i32 {
        self.min.y
    }

    /// Get a new rectangle with the same dimensions, but with the maximum point
    /// increased by one in order to treat the top and right edges inclusively.
    #[inline]
    #[must_use]
    pub fn inclusive(&self) -> Self {
        Self {
            min: self.min,
            max: self.max + IVec2::ONE,
        }
    }

    /// Get a new rectangle that is grown in all directions by the given amount.
    #[inline]
    #[must_use]
    pub fn grow(&self, amount: i32) -> Self {
        Self {
            min: self.min - IVec2::new(amount, amount),
            max: self.max + IVec2::new(amount, amount),
        }
    }

    /// Determine if this rectangle contains the given point.
    #[inline]
    #[must_use]
    pub fn contains<P>(&self, point: P) -> bool
    where
        P: Into<IVec2>,
    {
        let point = point.into();
        point.x >= self.left_bounds()
            && point.x <= self.right_bounds()
            && point.y <= self.top_bounds()
            && point.y >= self.bottom_bounds()
    }

    /// Get the minimum distance squared from the closest edge of this rectangle to the given point.
    #[inline]
    #[must_use]
    pub fn distance_squared_to<P>(&self, point: P) -> f32
    where
        P: Into<IVec2>,
    {
        let point = point.into();

        if self.contains(point) {
            return 0f32;
        }

        let dx = if point.x < self.min.x {
            self.min.x - point.x
        } else if point.x > self.max.x {
            point.x - self.max.x
        } else {
            0
        };
        let dy = if point.y < self.min.y {
            self.min.y - point.y
        } else if point.y > self.max.y {
            point.y - self.max.y
        } else {
            0
        };

        (dx * dx + dy * dy) as f32
    }

    /// Get the minimum distance from the closest edge of this rectangle to the given point.
    #[inline]
    #[must_use]
    pub fn distance_to<P>(&self, point: P) -> f32
    where
        P: Into<IVec2>,
    {
        self.distance_squared_to(point).sqrt()
    }

    /// Measure the axis-aligned distance from the given point to the edge's ray indicated by the
    /// given direction.
    ///
    /// # Parameters
    ///
    /// * `point`: The point from which to measure.
    /// * `direction`: A direction representing an edge of the rectangle, for example,
    ///   `Direction::North` for the top edge.
    ///
    /// # Panics
    ///
    /// Panics if the given direction is not a cardinal direction.
    #[inline]
    #[must_use]
    pub fn distance_to_edge<P>(&self, point: P, direction: Direction) -> i32
    where
        P: Into<IVec2>,
    {
        let point = point.into();
        match direction {
            Direction::North => (self.min.y - point.y).abs(),
            Direction::South => (point.y - self.max.y).abs(),
            Direction::East => (self.min.x - point.x).abs(),
            Direction::West => (point.x - self.max.x).abs(),
            _ => panic!("invalid direction: {:?}", direction),
        }
    }

    /// Ensure the given point is within this rectangle.
    #[inline]
    #[must_use]
    pub fn clamp(&self, point: IVec2) -> IVec2 {
        let mut point = point;
        if point.x < self.min.x {
            point.x = self.min.x;
        }
        if point.x > self.max.x {
            point.x = self.max.x;
        }
        if point.y < self.min.y {
            point.y = self.min.y;
        }
        if point.y > self.max.y {
            point.y = self.max.y;
        }
        point
    }

    /// Determine if this rectangle contains the given rectangle.
    #[inline]
    #[must_use]
    pub fn contains_rect(&self, rect: &Self) -> bool {
        self.contains(rect.min()) && self.contains(rect.max)
    }

    /// Determine if this rectangle intersects the given rectangle.
    #[inline]
    #[must_use]
    pub fn intersects_rect(&self, other: &Self) -> bool {
        if self.right_bounds() < other.left_bounds() || self.left_bounds() > other.right_bounds() {
            return false;
        }
        if self.top_bounds() < other.bottom_bounds() || self.bottom_bounds() > other.top_bounds() {
            return false;
        }
        true
    }

    /// Create a new rectangle that encompasses both this rectangle and the given rectangle.
    #[inline]
    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    /// Create a new rectangle that encompasses both this rectangle and the given point.
    #[inline]
    #[must_use]
    pub fn union_point(&self, other: IVec2) -> Self {
        Self {
            min: self.min.min(other),
            max: self.max.max(other),
        }
    }

    /// Create a new rectangle that is the intersection of this rectangle and the given rectangle.
    #[inline]
    #[must_use]
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let mut r = IRect {
            min: self.min.max(other.min),
            max: self.max.min(other.max),
        };
        r.min = r.min.min(r.max);
        if r.is_empty() {
            None
        } else {
            Some(r)
        }
    }

    /// Get the four corners of this rectangle, in #{Quadrant} order.
    #[inline]
    #[must_use]
    pub fn points(&self) -> [IVec2; 4] {
        [
            self.min,
            self.min + IVec2::new(self.width() as i32, 0),
            self.max,
            self.min + IVec2::new(0, self.height() as i32),
        ]
    }

    /// Get the four lines that make up the edges of this rectangle.
    #[inline]
    #[must_use]
    pub fn segments(&self) -> [ILine; 4] {
        let width = self.max.x - self.min.x;
        let height = self.max.y - self.min.y;
        [
            ILine::new(self.min, self.min + IVec2::new(width, 0)),
            ILine::new(self.min + IVec2::new(width, 0), self.max),
            ILine::new(self.max, self.min + IVec2::new(0, height)),
            ILine::new(self.min + IVec2::new(0, height), self.min),
        ]
    }

    #[inline]
    pub fn append_trimesh_data(
        &self,
        vertices: &mut Vec<IVec2>,
        indices: &mut Vec<u32>,
        offset: IVec2,
    ) {
        let index = vertices.len() as u32;
        vertices.extend([
            self.min + offset,
            IVec2::new(self.max.x, self.min.y) + offset,
            self.max + offset,
            IVec2::new(self.min.x, self.max.y) + offset,
        ]);

        indices.extend([index, index + 1, index + 2, index, index + 2, index + 3]);
    }

    #[inline]
    pub fn append_polyline_data(
        &self,
        vertices: &mut Vec<IVec2>,
        indices: &mut Vec<[u32; 2]>,
        offset: IVec2,
    ) {
        let index = vertices.len() as u32;
        vertices.extend([
            self.min + offset,
            IVec2::new(self.max.x, self.min.y) + offset,
            self.max + offset,
            IVec2::new(self.min.x, self.max.y) + offset,
        ]);

        indices.extend([
            [index, index + 1],
            [index + 1, index + 2],
            [index + 2, index + 3],
            [index + 3, index],
        ]);
    }

    #[inline]
    #[must_use]
    pub fn pixels(&self) -> RectPixelIterator {
        RectPixelIterator::new(self.clone())
    }
}

impl ops::Add<IVec2> for IRect {
    type Output = Self;

    #[inline]
    fn add(self, rhs: IVec2) -> Self::Output {
        Self::from_corners(self.min + rhs, self.max + rhs)
    }
}

impl ops::Add<IVec2> for &IRect {
    type Output = IRect;

    #[inline]
    fn add(self, rhs: IVec2) -> Self::Output {
        IRect::from_corners(self.min + rhs, self.max + rhs)
    }
}

impl ops::Sub<IVec2> for IRect {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: IVec2) -> Self::Output {
        Self::from_corners(self.min - rhs, self.max - rhs)
    }
}

impl ops::Sub<IVec2> for &IRect {
    type Output = IRect;

    #[inline]
    fn sub(self, rhs: IVec2) -> Self::Output {
        IRect::from_corners(self.min - rhs, self.max - rhs)
    }
}

impl<U: Unsigned + NumCast + Copy> From<Region<U>> for IRect {
    #[inline]
    fn from(region: Region<U>) -> Self {
        IRect::from(&region)
    }
}

impl<U: Unsigned + NumCast + Copy> From<&Region<U>> for IRect {
    #[inline]
    fn from(region: &Region<U>) -> Self {
        let size: i32 = num_traits::cast(region.size()).unwrap();
        let min = region.point();
        let max = min + size;
        Self::from_corners(min, max)
    }
}

impl From<ILine> for IRect {
    #[inline]
    fn from(line: ILine) -> Self {
        IRect::from(&line)
    }
}

impl From<&ILine> for IRect {
    #[inline]
    fn from(line: &ILine) -> Self {
        line.aabb()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RectPixelIterator {
    rect: IRect,
    x: i32,
    y: i32,
}

impl RectPixelIterator {
    #[inline]
    #[must_use]
    pub fn new(rect: IRect) -> Self {
        let x = rect.x();
        let y = rect.y();
        Self { rect, x, y }
    }
}

impl Iterator for RectPixelIterator {
    type Item = IVec2;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.x < self.rect.max.x {
            let x = self.x;
            self.x += 1;
            Some(IVec2::new(x, self.y))
        } else if self.y < self.rect.max.y - 1 {
            self.x = self.rect.min.x;
            self.y += 1;
            self.next()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_centered_at() {
        let rect = IRect::centered_at((1, 1), 2, 2);
        assert_eq!(rect.min.x, 0);
        assert_eq!(rect.min.y, 0);
        assert_eq!(rect.max.x, 2);
        assert_eq!(rect.max.y, 2);
    }

    #[test]
    fn test_contains() {
        let rect = IRect::new(1, 1, 3, 3);
        assert!(!rect.contains((0, 0)));
        assert!(rect.contains((1, 1)));
        assert!(rect.contains((1, 2)));
        assert!(!rect.contains((1, 3)));
        assert!(!rect.contains((1, 4)));
        assert!(rect.contains((2, 2)));
        assert!(!rect.contains((0, 0)));
        assert!(!rect.contains((3, 0)));
        assert!(!rect.contains((3, 3)));
        assert!(!rect.contains((4, 4)));
    }

    #[test]
    fn test_intersection() {
        let rect = IRect::new(1, 1, 3, 3);
        assert_eq!(
            rect.intersection(&IRect::new(1, 1, 3, 3)).unwrap(),
            IRect::new(1, 1, 3, 3)
        );
        assert_eq!(
            rect.intersection(&IRect::new(1, 1, 2, 2)).unwrap(),
            IRect::new(1, 1, 2, 2)
        );
        assert_eq!(
            rect.intersection(&IRect::new(2, 2, 1, 1)).unwrap(),
            IRect::new(2, 2, 1, 1)
        );
        assert_eq!(
            rect.intersection(&IRect::new(0, 0, 2, 2)).unwrap(),
            IRect::new(1, 1, 2, 2)
        );
        assert_eq!(
            rect.intersection(&IRect::new(0, 1, 2, 3)).unwrap(),
            IRect::new(1, 1, 2, 3)
        );
        assert_eq!(
            rect.intersection(&IRect::new(2, 1, 4, 3)).unwrap(),
            IRect::new(2, 1, 3, 3)
        );
        assert_eq!(
            rect.intersection(&IRect::new(1, 2, 3, 4)).unwrap(),
            IRect::new(1, 2, 3, 3)
        );
        assert_eq!(
            rect.intersection(&IRect::new(2, 2, 4, 4)).unwrap(),
            IRect::new(2, 2, 3, 3)
        );
        assert!(rect.intersection(&IRect::new(3, 3, 5, 5)).is_none());
    }

    #[test]
    fn test_pixels() {
        let rect = IRect::new(1, 1, 3, 3);
        let mut iter = rect.pixels();
        assert_eq!(iter.next(), Some((1, 1).into()));
        assert_eq!(iter.next(), Some((2, 1).into()));
        assert_eq!(iter.next(), Some((1, 2).into()));
        assert_eq!(iter.next(), Some((2, 2).into()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_intersects_rect() {
        let rect = IRect::new(1, 1, 3, 3);
        assert!(rect.intersects_rect(&IRect::new(1, 1, 3, 3)));
        assert!(rect.intersects_rect(&IRect::new(0, 0, 2, 2)));
        assert!(rect.intersects_rect(&IRect::new(0, 1, 2, 3)));
        assert!(rect.intersects_rect(&IRect::new(2, 1, 4, 3)));
        assert!(rect.intersects_rect(&IRect::new(1, 2, 3, 4)));
        assert!(rect.intersects_rect(&IRect::new(2, 2, 4, 4)));
        assert!(!rect.intersects_rect(&IRect::new(1, 3, 3, 5)));
        assert!(!rect.intersects_rect(&IRect::new(3, 1, 5, 3)));
        assert!(!rect.intersects_rect(&IRect::new(3, 3, 5, 5)));
    }

    #[test]
    fn test_distance_to() {
        let rect = IRect::new(1, 1, 3, 3);
        assert_eq!(rect.distance_to((0, 0)), std::f32::consts::SQRT_2);
        assert_eq!(rect.distance_to((1, 1)), 0.0);
        assert_eq!(rect.distance_to((2, 2)), 0.0);
        assert_eq!(rect.distance_to((3, 3)), 0.0);
        assert_eq!(rect.distance_to((4, 4)), std::f32::consts::SQRT_2);
        assert_eq!(rect.distance_to((5, 5)), std::f32::consts::SQRT_2 * 2.);
    }

    #[test]
    fn test_distance_to_edge() {
        let rect = IRect::new(1, 1, 3, 3);
        assert_eq!(rect.distance_to_edge((0, 0), Direction::East), 1);
        assert_eq!(rect.distance_to_edge((0, 0), Direction::North), 1);
        assert_eq!(rect.distance_to_edge((0, 0), Direction::West), 3);
        assert_eq!(rect.distance_to_edge((0, 0), Direction::South), 3);
        assert_eq!(rect.distance_to_edge((1, 1), Direction::East), 0);
        assert_eq!(rect.distance_to_edge((1, 1), Direction::North), 0);
        assert_eq!(rect.distance_to_edge((1, 1), Direction::West), 2);
        assert_eq!(rect.distance_to_edge((1, 1), Direction::South), 2);
        assert_eq!(rect.distance_to_edge((2, 2), Direction::East), 1);
        assert_eq!(rect.distance_to_edge((2, 2), Direction::North), 1);
        assert_eq!(rect.distance_to_edge((2, 2), Direction::West), 1);
        assert_eq!(rect.distance_to_edge((2, 2), Direction::South), 1);
    }

    #[test]
    fn test_append_trimesh_data() {
        let rect = IRect::new(1, 1, 3, 3);
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        rect.append_trimesh_data(&mut vertices, &mut indices, IVec2::default());
        assert_eq!(vertices.len(), 4);
        assert_eq!(indices.len(), 6);
        assert_eq!(vertices[0], (1, 1).into());
        assert_eq!(vertices[1], (3, 1).into());
        assert_eq!(vertices[2], (3, 3).into());
        assert_eq!(vertices[3], (1, 3).into());
        assert_eq!(indices[0], 0);
        assert_eq!(indices[1], 1);
        assert_eq!(indices[2], 2);
        assert_eq!(indices[3], 0);
        assert_eq!(indices[4], 2);
        assert_eq!(indices[5], 3);
    }

    #[test]
    fn test_append_polyline_data() {
        let rect = IRect::new(1, 1, 3, 3);
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        rect.append_polyline_data(&mut vertices, &mut indices, IVec2::default());
        assert_eq!(vertices.len(), 4);
        assert_eq!(vertices[0], (1, 1).into());
        assert_eq!(vertices[1], (3, 1).into());
        assert_eq!(vertices[2], (3, 3).into());
        assert_eq!(vertices[3], (1, 3).into());
        assert_eq!(indices.len(), 4);
        assert_eq!(indices[0], [0, 1]);
        assert_eq!(indices[1], [1, 2]);
        assert_eq!(indices[2], [2, 3]);
        assert_eq!(indices[3], [3, 0]);
    }
}
