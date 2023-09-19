#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::line_interval::LineInterval;
use super::line_iterator::{plot_line, LinePixelIterator};
use crate::{distance_squared_to, distance_to, urect_edges, Direction};
use bevy_math::{uvec2, URect, UVec2};

/// A line segment represented by two points, in integer coordinates.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ULine {
    start: UVec2,
    end: UVec2,
}

impl ULine {
    pub const ZERO: Self = Self {
        start: UVec2::ZERO,
        end: UVec2::ZERO,
    };

    /// Creates a new line with the given start and end points.
    #[inline]
    #[must_use]
    pub fn new<P>(start: P, end: P) -> Self
    where
        P: Into<UVec2>,
    {
        Self {
            start: start.into(),
            end: end.into(),
        }
    }

    /// Get the start point.
    #[inline]
    #[must_use]
    pub fn start(&self) -> UVec2 {
        self.start
    }

    /// Get the end point.
    #[inline]
    #[must_use]
    pub fn end(&self) -> UVec2 {
        self.end
    }

    /// Get the line's length squared.
    #[inline]
    #[must_use]
    pub fn length_squared(&self) -> f32 {
        distance_squared_to(self.start, self.end)
    }

    /// Get the line's length.
    #[inline]
    #[must_use]
    pub fn length(&self) -> f32 {
        distance_to(self.start, self.end)
    }

    /// Create a new line that is the rotation of this line around its start point, by the given radians.
    #[inline]
    #[must_use]
    pub fn rotate(&self, radians: f32) -> Self {
        self.rotate_around(self.start, radians)
    }

    /// Create a new line that is the rotation of this line around the given point, by the given radians.
    #[inline]
    #[must_use]
    pub fn rotate_around(&self, center: UVec2, radians: f32) -> Self {
        let cos_theta = f32::cos(radians);
        let sin_theta = f32::sin(radians);

        let start_x_diff = self.start.x as f32 - center.x as f32;
        let start_y_diff = self.start.y as f32 - center.y as f32;

        let end_x_diff = self.end.x as f32 - center.x as f32;
        let end_y_diff = self.end.y as f32 - center.y as f32;

        let x0 = cos_theta * start_x_diff - sin_theta * start_y_diff + center.x as f32;
        let y0 = sin_theta * start_x_diff + cos_theta * start_y_diff + center.y as f32;

        let x1 = cos_theta * end_x_diff - sin_theta * end_y_diff + center.x as f32;
        let y1 = sin_theta * end_x_diff + cos_theta * end_y_diff + center.y as f32;

        Self::new((x0 as u32, y0 as u32), (x1 as u32, y1 as u32))
    }

    /// Flip the orientation of the line such that the start point
    /// becomes the end point and vice versa.
    #[inline]
    #[must_use]
    pub fn flip(&self) -> Self {
        Self::new(self.end, self.start)
    }

    /// Determine if the given point lies on this line.
    #[inline]
    #[must_use]
    pub fn contains<P>(&self, point: P) -> bool
    where
        P: Into<UVec2>,
    {
        let point = point.into();
        let d = distance_to(self.start, point) + distance_to(point, self.end) - self.length();
        -f32::EPSILON < d && d < f32::EPSILON
    }

    #[inline]
    #[must_use]
    pub fn is_vertical(&self) -> bool {
        self.start.x == self.end.x
    }

    #[inline]
    pub fn is_horizontal(&self) -> bool {
        self.start.y == self.end.y
    }

    /// Determine if this line is axis-aligned.
    #[inline]
    #[must_use]
    pub fn is_axis_aligned(&self) -> bool {
        self.is_horizontal() || self.is_vertical()
    }

    /// Get the axis-aligned bounding box of this line.
    #[inline]
    #[must_use]
    pub fn aabb(&self) -> URect {
        URect::from_corners(self.start, self.end)
    }

    /// Get the axis-aligned direction of this line, if it is axis-aligned, `None` otherwise.
    #[inline]
    #[must_use]
    pub fn axis_alignment(&self) -> Option<Direction> {
        if self.start.x == self.end.x {
            if self.start.y < self.end.y {
                Some(Direction::North)
            } else {
                Some(Direction::South)
            }
        } else if self.start.y == self.end.y {
            if self.start.x > self.end.x {
                Some(Direction::West)
            } else {
                Some(Direction::East)
            }
        } else {
            None
        }
    }

    /// Get the diagonal axis-aligned direction of this line, if it is diagonally axis-aligned, `None` otherwise.
    #[inline]
    #[must_use]
    pub fn diagonal_axis_alignment(&self) -> Option<Direction> {
        let dx = self.end.x as i64 - self.start.x as i64;
        let dy = self.end.y as i64 - self.start.y as i64;
        if dx == dy {
            if dx > 0 {
                Some(Direction::NorthEast)
            } else {
                Some(Direction::SouthWest)
            }
        } else if dx == -dy {
            if dx > 0 {
                Some(Direction::SouthEast)
            } else {
                Some(Direction::NorthWest)
            }
        } else {
            None
        }
    }

    /// Determine if this line intersects the given line.
    #[inline]
    #[must_use]
    pub fn intersects_line(&self, other: &ULine) -> Option<UVec2> {
        let seg1 = LineInterval::line_segment(*self);
        let seg2 = LineInterval::line_segment(*other);
        seg1.relate(&seg2).unique_intersection()
    }

    /// Determine if this line intersects the given rectangle.
    #[inline]
    #[must_use]
    pub fn intersects_rect(&self, rect: &URect) -> bool {
        for seg in urect_edges(rect) {
            if self.intersects_line(&seg).is_some() {
                return true;
            }
        }
        false
    }

    /// Obtain the segment of this line that intersects the given rectangle, if any, otherwise `None`.
    /// This line must be axis-aligned, otherwise `None` is returned.
    #[inline]
    #[must_use]
    pub fn axis_aligned_intersect_rect(&self, rect: &URect) -> Option<ULine> {
        if self.is_vertical() {
            // Ensure the smaller `y` value is at the bottom
            let (start, end, flipped) = if self.start.y < self.end.y {
                (self.start, self.end, false)
            } else {
                (self.end, self.start, true)
            };

            // Ensure the vertical line is within the horizontal bounds of the rect
            if start.x < rect.min.x || start.x > rect.max.x {
                return None;
            }

            // Restrain the `y` values to the rect
            let start_y = start.y.max(rect.min.y).min(rect.max.y);
            let end_y = end.y.max(rect.min.y).min(rect.max.y);

            if start_y <= end_y {
                let result = ULine::new((start.x, start_y), (start.x, end_y));
                if flipped {
                    Some(result.flip())
                } else {
                    Some(result)
                }
            } else {
                None
            }
        } else if self.is_horizontal() {
            // Ensure the smaller `x` value is at the left
            let (start, end, flipped) = if self.start.x < self.end.x {
                (self.start, self.end, false)
            } else {
                (self.end, self.start, true)
            };

            // Ensure the horizontal line is within the vertical bounds of the rect
            if start.y < rect.min.y || start.y > rect.max.y {
                return None;
            }

            // Restrain the `x` values to the rect
            let start_x = start.x.max(rect.min.x).min(rect.max.x);
            let end_x = end.x.max(rect.min.x).min(rect.max.x);

            if start_x <= end_x {
                let result = ULine::new((start_x, start.y), (end_x, start.y));
                if flipped {
                    Some(result.flip())
                } else {
                    Some(result)
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// If this and the given line segments overlap, return the overlapping segment.
    /// Otherwise, return `None`.
    #[inline]
    #[must_use]
    pub fn overlap(&self, other: &ULine) -> Option<ULine> {
        // Check if the edges share a common point
        if self.start == other.start
            || self.start == other.end
            || self.end == other.start
            || self.end == other.end
        {
            // If they share a common point, return the shorter of the two edges
            let self_length = self.length_squared();
            let other_length = other.length_squared();
            return if self_length < other_length {
                Some(*self)
            } else {
                Some(*other)
            };
        }

        // Check for overlapping segments
        let min_x1 = self.start.x.min(self.end.x);
        let max_x1 = self.start.x.max(self.end.x);
        let min_y1 = self.start.y.min(self.end.y);
        let max_y1 = self.start.y.max(self.end.y);

        let min_x2 = other.start.x.min(other.end.x);
        let max_x2 = other.start.x.max(other.end.x);
        let min_y2 = other.start.y.min(other.end.y);
        let max_y2 = other.start.y.max(other.end.y);

        if max_x1 < min_x2 || min_x1 > max_x2 || max_y1 < min_y2 || min_y1 > max_y2 {
            // No overlap
            return None;
        }

        // Calculate the overlapping segment
        let overlap_start_x = max_x1.min(max_x2);
        let overlap_start_y = max_y1.min(max_y2);
        let overlap_end_x = min_x1.max(min_x2);
        let overlap_end_y = min_y1.max(min_y2);

        let overlap_start = uvec2(overlap_start_x, overlap_start_y);
        let overlap_end = uvec2(overlap_end_x, overlap_end_y);

        Some(ULine::new(overlap_start, overlap_end))
    }

    /// Use Bresenham's line algorithm to visit points on this line.
    #[inline]
    pub fn visit_points<F>(&self, mut visitor: F)
    where
        F: FnMut(u32, u32),
    {
        plot_line(
            self.start.x as i32,
            self.start.y as i32,
            self.end.x as i32,
            self.end.y as i32,
            |x, y| {
                visitor(x as u32, y as u32);
            },
        );
    }

    #[inline]
    #[must_use]
    pub fn pixels(&self) -> LinePixelIterator {
        LinePixelIterator::new(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_contains() {
        let line = ULine::new((0, 0), (10, 10));
        assert!(line.contains((5, 5)));
        assert!(!line.contains((5, 6)));
        assert!(!line.contains((6, 5)));
    }

    #[test]
    fn test_aabb() {
        let line = ULine::new((0, 0), (10, 10));
        let aabb = line.aabb();
        assert_eq!(aabb.min.x, 0);
        assert_eq!(aabb.min.y, 0);
        assert_eq!(aabb.width(), 10);
        assert_eq!(aabb.height(), 10);
    }

    #[test]
    fn test_axis_alignment() {
        let line = ULine::new((0, 0), (10, 10));
        assert_eq!(line.axis_alignment(), None);
        let line = ULine::new((0, 0), (10, 0));
        assert_eq!(line.axis_alignment(), Some(Direction::East));
        let line = ULine::new((0, 0), (0, 10));
        assert_eq!(line.axis_alignment(), Some(Direction::North));
        let line = ULine::new((10, 0), (0, 0));
        assert_eq!(line.axis_alignment(), Some(Direction::West));
        let line = ULine::new((0, 10), (0, 0));
        assert_eq!(line.axis_alignment(), Some(Direction::South));
        let line = ULine::new((0, 10), (1, 0));
        assert_eq!(line.axis_alignment(), None);
    }

    #[test]
    fn test_diag_axis_alignment() {
        let line = ULine::new((0, 0), (9, 10));
        assert_eq!(line.diagonal_axis_alignment(), None);
        let line = ULine::new((0, 0), (10, 10));
        assert_eq!(line.diagonal_axis_alignment(), Some(Direction::NorthEast));
        let line = ULine::new((0, 10), (10, 0));
        assert_eq!(line.diagonal_axis_alignment(), Some(Direction::SouthEast));
        let line = ULine::new((10, 0), (0, 10));
        assert_eq!(line.diagonal_axis_alignment(), Some(Direction::NorthWest));
        let line = ULine::new((10, 10), (0, 0));
        assert_eq!(line.diagonal_axis_alignment(), Some(Direction::SouthWest));
    }
}
