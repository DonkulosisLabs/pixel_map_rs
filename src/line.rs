use super::line_interval::LineInterval;
use super::line_iterator::{plot_line, LineIterator};
use super::{Direction, IRect, Point};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Line {
    start: Point,
    end: Point,
}

impl Line {
    #[inline]
    pub fn new<P>(start: P, end: P) -> Self
    where
        P: Into<Point>,
    {
        Self {
            start: start.into(),
            end: end.into(),
        }
    }

    #[inline]
    pub fn start(&self) -> Point {
        self.start
    }

    #[inline]
    pub fn end(&self) -> Point {
        self.end
    }

    #[inline]
    pub fn length_squared(&self) -> f32 {
        self.start.distance_squared_to(self.end)
    }

    #[inline]
    pub fn length(&self) -> f32 {
        self.start.distance_to(self.end)
    }

    #[inline]
    pub fn rotate(&self, radians: f32) -> Self {
        self.rotate_around(self.start, radians)
    }

    pub fn rotate_around(&self, center: Point, radians: f32) -> Self {
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

        Self::new((x0 as i32, y0 as i32), (x1 as i32, y1 as i32))
    }

    #[inline]
    pub fn contains<P>(&self, point: P) -> bool
    where
        P: Into<Point>,
    {
        let point = point.into();
        let d = self.start.distance_to(point) + point.distance_to(self.end) - self.length();
        -f32::EPSILON < d && d < f32::EPSILON
    }

    #[inline]
    pub fn is_axis_aligned(&self) -> bool {
        self.start.x() == self.end.x() || self.start.y() == self.end.y()
    }

    #[inline]
    pub fn aabb(&self) -> IRect {
        IRect::from_corners(self.start, self.end)
    }

    #[inline]
    pub fn axis_alignment(&self) -> Option<Direction> {
        if self.start.x() == self.end.x() {
            if self.start.y() < self.end.y() {
                Some(Direction::North)
            } else {
                Some(Direction::South)
            }
        } else if self.start.y() == self.end.y() {
            if self.start.x() > self.end.x() {
                Some(Direction::West)
            } else {
                Some(Direction::East)
            }
        } else {
            None
        }
    }

    #[inline]
    pub fn diagonal_axis_alignment(&self) -> Option<Direction> {
        let dx = self.end.x() - self.start.x();
        let dy = self.end.y() - self.start.y();
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

    #[inline]
    pub fn intersects_line(&self, other: &Line) -> Option<Point> {
        let seg1 = LineInterval::line_segment(*self);
        let seg2 = LineInterval::line_segment(*other);
        seg1.relate(&seg2).unique_intersection()
    }

    pub fn intersects_rect(&self, rect: &IRect) -> bool {
        for seg in rect.segments() {
            if self.intersects_line(&seg).is_some() {
                return true;
            }
        }
        false
    }

    /// Use Bresenham's line algorithm to visit points on this line.
    #[inline]
    pub fn visit_points<F>(&self, visitor: F)
    where
        F: FnMut(i32, i32),
    {
        plot_line(
            self.start.x(),
            self.start.y(),
            self.end.x(),
            self.end.y(),
            visitor,
        );
    }

    pub fn iter(&self) -> LineIterator {
        LineIterator::new(&self)
    }
}

impl IntoIterator for Line {
    type Item = Point;
    type IntoIter = LineIterator;

    fn into_iter(self) -> Self::IntoIter {
        LineIterator::new(&self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_contains() {
        let line = Line::new((0, 0), (10, 10));
        assert!(line.contains((5, 5)));
        assert!(!line.contains((5, 6)));
        assert!(!line.contains((6, 5)));
    }

    #[test]
    fn test_aabb() {
        let line = Line::new((0, 0), (10, 10));
        let aabb = line.aabb();
        assert_eq!(aabb.x(), 0);
        assert_eq!(aabb.y(), 0);
        assert_eq!(aabb.width(), 10);
        assert_eq!(aabb.height(), 10);
    }

    #[test]
    fn test_axis_alignment() {
        let line = Line::new((0, 0), (10, 10));
        assert_eq!(line.axis_alignment(), None);
        let line = Line::new((0, 0), (10, 0));
        assert_eq!(line.axis_alignment(), Some(Direction::East));
        let line = Line::new((0, 0), (0, 10));
        assert_eq!(line.axis_alignment(), Some(Direction::North));
        let line = Line::new((10, 0), (0, 0));
        assert_eq!(line.axis_alignment(), Some(Direction::West));
        let line = Line::new((0, 10), (0, 0));
        assert_eq!(line.axis_alignment(), Some(Direction::South));
        let line = Line::new((0, 10), (1, 0));
        assert_eq!(line.axis_alignment(), None);
    }

    #[test]
    fn test_diag_axis_alignment() {
        let line = Line::new((0, 0), (9, 10));
        assert_eq!(line.diagonal_axis_alignment(), None);
        let line = Line::new((0, 0), (10, 10));
        assert_eq!(line.diagonal_axis_alignment(), Some(Direction::NorthEast));
        let line = Line::new((0, 10), (10, 0));
        assert_eq!(line.diagonal_axis_alignment(), Some(Direction::SouthEast));
        let line = Line::new((10, 0), (0, 10));
        assert_eq!(line.diagonal_axis_alignment(), Some(Direction::NorthWest));
        let line = Line::new((10, 10), (0, 0));
        assert_eq!(line.diagonal_axis_alignment(), Some(Direction::SouthWest));
    }
}
