use super::region::Region;
use super::Line;
use glam::IVec2;

use num_traits::{NumCast, Unsigned};
use std::ops;

/// An immutable rectangle defined by a minimum and maximum point, in integer coordinates.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IRect {
    min: IVec2,
    max: IVec2,
}

impl Default for IRect {
    fn default() -> Self {
        Self::ZERO
    }
}

impl IRect {
    pub const ZERO: Self = Self {
        min: IVec2::ZERO,
        max: IVec2::ZERO,
    };
    pub const ONE: Self = Self {
        min: IVec2::ONE,
        max: IVec2::ONE,
    };
    pub const NEG_ONE: Self = Self {
        min: IVec2::NEG_ONE,
        max: IVec2::NEG_ONE,
    };

    #[inline]
    pub fn new(x0: i32, y0: i32, x1: i32, y1: i32) -> Self {
        Self::from_corners((x0, y0), (x1, y1))
    }

    #[inline]
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

    /// Create a new {Self} with the given center point, and width and height.
    #[inline]
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

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.min.x == self.max.x && self.min.y == self.max.y
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.min.x == self.max.x || self.min.y == self.max.y
    }

    #[inline]
    pub fn x(&self) -> i32 {
        self.min.x
    }

    #[inline]
    pub fn y(&self) -> i32 {
        self.min.y
    }

    #[inline]
    pub fn min(&self) -> IVec2 {
        self.min
    }

    #[inline]
    pub fn max(&self) -> IVec2 {
        self.max
    }

    #[inline]
    pub fn center(&self) -> IVec2 {
        let half_max = self.max / 2;
        self.min + half_max
    }

    #[inline]
    pub fn width(&self) -> u32 {
        (self.max.x - self.min.x) as u32
    }

    #[inline]
    pub fn height(&self) -> u32 {
        (self.max.y - self.min.y) as u32
    }

    #[inline]
    pub fn size(&self) -> IVec2 {
        self.max - self.min
    }

    #[inline]
    pub fn left_bounds(&self) -> i32 {
        self.min.x
    }

    #[inline]
    pub fn right_bounds(&self) -> i32 {
        self.max.x - 1
    }

    #[inline]
    pub fn top_bounds(&self) -> i32 {
        self.max.y - 1
    }

    #[inline]
    pub fn bottom_bounds(&self) -> i32 {
        self.min.y
    }

    #[inline]
    pub fn inclusive(&self) -> Self {
        Self {
            min: self.min,
            max: self.max + IVec2::ONE,
        }
    }

    #[inline]
    pub fn grow(&self, amount: i32) -> Self {
        Self {
            min: self.min - IVec2::new(amount, amount),
            max: self.max + IVec2::new(amount, amount),
        }
    }

    #[inline]
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

    #[inline]
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

    #[inline]
    pub fn distance_to<P>(&self, point: P) -> f32
    where
        P: Into<IVec2>,
    {
        self.distance_squared_to(point).sqrt()
    }

    #[inline]
    pub fn contains_rect(&self, rect: &Self) -> bool {
        self.contains(rect.min()) && self.contains(rect.max)
    }

    #[inline]
    pub fn intersects_rect(&self, other: &Self) -> bool {
        if self.right_bounds() < other.left_bounds() || self.left_bounds() > other.right_bounds() {
            return false;
        }
        if self.top_bounds() < other.bottom_bounds() || self.bottom_bounds() > other.top_bounds() {
            return false;
        }
        true
    }

    #[inline]
    pub fn union(&self, other: &Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    #[inline]
    pub fn union_point(&self, other: IVec2) -> Self {
        Self {
            min: self.min.min(other),
            max: self.max.max(other),
        }
    }

    #[inline]
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

    #[inline]
    pub fn segments(&self) -> [Line; 4] {
        let width = self.max.x - self.min.x;
        let height = self.max.y - self.min.y;
        [
            Line::new(self.min, self.min + IVec2::new(width, 0)),
            Line::new(self.min + IVec2::new(width, 0), self.max),
            Line::new(self.max, self.min + IVec2::new(0, height)),
            Line::new(self.min + IVec2::new(0, height), self.min),
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
}

impl ops::Add<IVec2> for IRect {
    type Output = Self;

    fn add(self, rhs: IVec2) -> Self::Output {
        Self::from_corners(self.min + rhs, self.max + rhs)
    }
}

impl ops::Sub<IVec2> for IRect {
    type Output = Self;

    fn sub(self, rhs: IVec2) -> Self::Output {
        Self::from_corners(self.min - rhs, self.max - rhs)
    }
}

impl<U: Unsigned + NumCast + Copy> From<Region<U>> for IRect {
    fn from(region: Region<U>) -> Self {
        IRect::from(&region)
    }
}

impl<U: Unsigned + NumCast + Copy> From<&Region<U>> for IRect {
    fn from(region: &Region<U>) -> Self {
        let size: i32 = num_traits::cast(region.size()).unwrap();
        let min = region.point();
        let max = min + size;
        Self::from_corners(min, max)
    }
}

impl From<Line> for IRect {
    fn from(line: Line) -> Self {
        IRect::from(&line)
    }
}

impl From<&Line> for IRect {
    fn from(line: &Line) -> Self {
        line.aabb()
    }
}

impl IntoIterator for IRect {
    type Item = (i32, i32);
    type IntoIter = RectPixelIterator;

    fn into_iter(self) -> Self::IntoIter {
        let x = self.x();
        let y = self.y();
        RectPixelIterator { rect: self, x, y }
    }
}

pub struct RectPixelIterator {
    rect: IRect,
    x: i32,
    y: i32,
}

impl Iterator for RectPixelIterator {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.x < self.rect.max.x {
            let x = self.x;
            self.x += 1;
            Some((x, self.y))
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
    fn test_into_iter() {
        let rect = IRect::new(1, 1, 3, 3);
        let mut iter = rect.into_iter();
        assert_eq!(iter.next(), Some((1, 1)));
        assert_eq!(iter.next(), Some((2, 1)));
        assert_eq!(iter.next(), Some((1, 2)));
        assert_eq!(iter.next(), Some((2, 2)));
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
        assert_eq!(rect.distance_to((0, 0)), 1.4142135);
        assert_eq!(rect.distance_to((1, 1)), 0.0);
        assert_eq!(rect.distance_to((2, 2)), 0.0);
        assert_eq!(rect.distance_to((3, 3)), 0.0);
        assert_eq!(rect.distance_to((4, 4)), 1.4142135);
        assert_eq!(rect.distance_to((5, 5)), 2.828427);
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
