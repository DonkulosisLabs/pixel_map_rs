#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::ULine;
use crate::{exclusive_urect, Direction};
use bevy_math::{ivec2, uvec2, IVec2, URect, UVec2};

pub fn plot_line<F>(x0: i32, y0: i32, x1: i32, y1: i32, mut plot: F)
where
    F: FnMut(i32, i32),
{
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let mut x = x0;
    let mut y = y0;
    let mut xi = 1;
    let mut yi = 1;

    if x1 < x0 {
        xi = -1;
    }

    if y1 < y0 {
        yi = -1;
    }

    let mut err = dx - dy;
    let mut e2: i32;

    while x != x1 || y != y1 {
        plot(x, y);
        e2 = err * 2;
        if e2 > -dy {
            err -= dy;
            x += xi;
        }
        if e2 < dx {
            err += dx;
            y += yi;
        }
    }

    plot(x1, y1);
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum LinePixelIterator {
    Axis(AxisLineIterator),
    Angle(AngleLineIterator),
}

impl LinePixelIterator {
    #[inline]
    #[must_use]
    pub fn new(line: &ULine) -> Self {
        match AxisLineIterator::new(line) {
            Some(iter) => LinePixelIterator::Axis(iter),
            None => LinePixelIterator::Angle(AngleLineIterator::new(line)),
        }
    }

    #[inline]
    #[must_use]
    pub fn peek(&self) -> Option<UVec2> {
        match self {
            LinePixelIterator::Axis(iter) => iter.peek(),
            LinePixelIterator::Angle(iter) => iter.peek(),
        }
    }

    /// Seek the iterator to the first point on the line that is on the given bounds, and return it.
    /// Calling next() after this will return the point beyond the bounds, if there is one
    /// for the line segment.
    /// Returns None if the end of the iterator is reached without touching the bounds.
    #[inline]
    pub fn seek_bounds(&mut self, bounds: &URect) -> Option<UVec2> {
        match self {
            LinePixelIterator::Axis(iter) => iter.seek_bounds(bounds),
            LinePixelIterator::Angle(iter) => iter.seek_bounds(bounds),
        }
    }
}

impl Iterator for LinePixelIterator {
    type Item = UVec2;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            LinePixelIterator::Axis(iter) => iter.next(),
            LinePixelIterator::Angle(iter) => iter.next(),
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct AxisLineIterator {
    point: UVec2,
    direction: Direction,
    end: UVec2,
    finished: bool,
}

impl AxisLineIterator {
    #[inline]
    #[must_use]
    pub fn new(line: &ULine) -> Option<Self> {
        let direction = line.axis_alignment().or(line.diagonal_axis_alignment())?;
        Some(Self {
            point: line.start(),
            direction,
            end: line.end(),
            finished: false,
        })
    }

    #[inline]
    #[must_use]
    pub fn peek(&self) -> Option<UVec2> {
        if self.finished {
            return None;
        }
        Some(self.point)
    }

    pub fn seek_bounds(&mut self, bounds: &URect) -> Option<UVec2> {
        let point = self.next()?;

        let top = (bounds.max.y - 1).min(self.end.y);
        let left = bounds.min.x.max(self.end.x);
        let right = (bounds.max.x - 1).min(self.end.x);
        let bottom = bounds.min.y.max(self.end.y);

        let result = match self.direction {
            Direction::North => Some(uvec2(point.x, top)),
            Direction::NorthEast => Some(uvec2(right, top)),
            Direction::NorthWest => {
                // account for the left being inclusive and the top being exclusive
                let left = (bounds.min.x + 1).max(self.end.x);
                Some(uvec2(left, top))
            }
            Direction::East => Some(uvec2(right, point.y)),
            Direction::South => Some(uvec2(point.x, bottom)),
            Direction::SouthEast => {
                // account for the right being exclusive and the bottom being inclusive
                let bottom = (bounds.min.y + 1).max(self.end.y);
                Some(uvec2(right, bottom))
            }
            Direction::SouthWest => Some(uvec2(left, bottom)),
            Direction::West => Some(uvec2(left, point.y)),
        };

        match result {
            Some(point) => {
                // Move the iterator to the calculated point
                self.point = point;

                // Consume the point
                self.next()
            }
            None => None,
        }
    }
}

impl Iterator for AxisLineIterator {
    type Item = UVec2;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            None
        } else {
            let result = self.point;
            if self.point == self.end {
                self.finished = true;
            } else {
                self.point = self.direction.move_point(self.point, 1);
            }
            Some(result)
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct AngleLineIterator {
    end: IVec2,
    dist: IVec2,
    point: IVec2,
    xi: i32,
    yi: i32,
    err: i32,
    e2: i32,
    finished: bool,
}

impl AngleLineIterator {
    #[inline]
    #[must_use]
    pub fn new(line: &ULine) -> Self {
        let start = line.start().as_ivec2();
        let end = line.end().as_ivec2();
        let x0 = start.x;
        let y0 = start.y;
        let x1 = end.x;
        let y1 = end.y;
        let dist = ivec2((x1 - x0).abs(), (y1 - y0).abs());
        let xi = if x1 < x0 { -1 } else { 1 };
        let yi = if y1 < y0 { -1 } else { 1 };
        AngleLineIterator {
            end: line.end().as_ivec2(),
            dist,
            point: line.start().as_ivec2(),
            xi,
            yi,
            err: dist.x - dist.y,
            e2: 0,
            finished: false,
        }
    }

    #[inline]
    #[must_use]
    pub fn peek(&self) -> Option<UVec2> {
        if self.finished {
            return None;
        }
        Some(self.point.as_uvec2())
    }

    #[inline]
    pub fn seek_bounds(&mut self, bounds: &URect) -> Option<UVec2> {
        let bounds = exclusive_urect(bounds);
        while let Some(point) = self.next() {
            if let Some(next) = self.peek() {
                if !bounds.contains(next) {
                    return Some(point);
                }
            } else {
                return Some(point);
            }
        }
        None
    }
}

impl Iterator for AngleLineIterator {
    type Item = UVec2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            None
        } else {
            let result = self.point;
            if self.point == self.end {
                self.finished = true;
            } else {
                self.e2 = self.err * 2;
                if self.e2 > -self.dist.y {
                    self.err -= self.dist.y;
                    self.point += ivec2(self.xi, 0);
                }
                if self.e2 < self.dist.x {
                    self.err += self.dist.x;
                    self.point += ivec2(0, self.yi);
                }
            }
            Some(result.as_uvec2())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{EAST, NORTH, NORTH_EAST, NORTH_WEST, SOUTH, SOUTH_EAST, SOUTH_WEST, WEST};
    use bevy_math::IVec2;

    #[test]
    fn test_plot_line_north() {
        let mut points = Vec::new();
        plot_line(0, 0, 0, 10, |x, y| points.push((x, y)));
        assert_eq!(points.len(), 11);
        assert_eq!(points[0], (0, 0));
        assert_eq!(points[1], (0, 1));
        assert_eq!(points[2], (0, 2));
        assert_eq!(points[3], (0, 3));
        assert_eq!(points[4], (0, 4));
        assert_eq!(points[5], (0, 5));
        assert_eq!(points[6], (0, 6));
        assert_eq!(points[7], (0, 7));
        assert_eq!(points[8], (0, 8));
        assert_eq!(points[9], (0, 9));
        assert_eq!(points[10], (0, 10));
    }

    #[test]
    fn test_plot_line_nw() {
        let mut points = Vec::new();
        plot_line(0, 0, 10, 10, |x, y| points.push((x, y)));
        assert_eq!(points.len(), 11);
        assert_eq!(points[0], (0, 0));
        assert_eq!(points[1], (1, 1));
        assert_eq!(points[2], (2, 2));
        assert_eq!(points[3], (3, 3));
        assert_eq!(points[4], (4, 4));
        assert_eq!(points[5], (5, 5));
        assert_eq!(points[6], (6, 6));
        assert_eq!(points[7], (7, 7));
        assert_eq!(points[8], (8, 8));
        assert_eq!(points[9], (9, 9));
        assert_eq!(points[10], (10, 10));
    }

    #[test]
    fn test_plot_line_ne() {
        let mut points = Vec::new();
        plot_line(10, 0, 0, 10, |x, y| points.push((x, y)));
        assert_eq!(points.len(), 11);
        assert_eq!(points[0], (10, 0));
        assert_eq!(points[1], (9, 1));
        assert_eq!(points[2], (8, 2));
        assert_eq!(points[3], (7, 3));
        assert_eq!(points[4], (6, 4));
        assert_eq!(points[5], (5, 5));
        assert_eq!(points[6], (4, 6));
        assert_eq!(points[7], (3, 7));
        assert_eq!(points[8], (2, 8));
        assert_eq!(points[9], (1, 9));
        assert_eq!(points[10], (0, 10));
    }

    #[test]
    fn test_plot_line_west() {
        let mut points = Vec::new();
        plot_line(10, 0, 0, 0, |x, y| points.push((x, y)));
        assert_eq!(points.len(), 11);
        assert_eq!(points[0], (10, 0));
        assert_eq!(points[1], (9, 0));
        assert_eq!(points[2], (8, 0));
        assert_eq!(points[3], (7, 0));
        assert_eq!(points[4], (6, 0));
        assert_eq!(points[5], (5, 0));
        assert_eq!(points[6], (4, 0));
        assert_eq!(points[7], (3, 0));
        assert_eq!(points[8], (2, 0));
        assert_eq!(points[9], (1, 0));
        assert_eq!(points[10], (0, 0));
    }

    #[test]
    fn test_plot_line_east() {
        let mut points = Vec::new();
        plot_line(0, 0, 10, 0, |x, y| points.push((x, y)));
        assert_eq!(points.len(), 11);
        assert_eq!(points[0], (0, 0));
        assert_eq!(points[1], (1, 0));
        assert_eq!(points[2], (2, 0));
        assert_eq!(points[3], (3, 0));
        assert_eq!(points[4], (4, 0));
        assert_eq!(points[5], (5, 0));
        assert_eq!(points[6], (6, 0));
        assert_eq!(points[7], (7, 0));
        assert_eq!(points[8], (8, 0));
        assert_eq!(points[9], (9, 0));
        assert_eq!(points[10], (10, 0));
    }

    #[test]
    fn test_plot_line_sw() {
        let mut points = Vec::new();
        plot_line(0, 10, 10, 0, |x, y| points.push((x, y)));
        assert_eq!(points.len(), 11);
        assert_eq!(points[0], (0, 10));
        assert_eq!(points[1], (1, 9));
        assert_eq!(points[2], (2, 8));
        assert_eq!(points[3], (3, 7));
        assert_eq!(points[4], (4, 6));
        assert_eq!(points[5], (5, 5));
        assert_eq!(points[6], (6, 4));
        assert_eq!(points[7], (7, 3));
        assert_eq!(points[8], (8, 2));
        assert_eq!(points[9], (9, 1));
        assert_eq!(points[10], (10, 0));
    }

    #[test]
    fn test_plot_line_se() {
        let mut points = Vec::new();
        plot_line(10, 10, 0, 0, |x, y| points.push((x, y)));
        assert_eq!(points.len(), 11);
        assert_eq!(points[0], (10, 10));
        assert_eq!(points[1], (9, 9));
        assert_eq!(points[2], (8, 8));
        assert_eq!(points[3], (7, 7));
        assert_eq!(points[4], (6, 6));
        assert_eq!(points[5], (5, 5));
        assert_eq!(points[6], (4, 4));
        assert_eq!(points[7], (3, 3));
        assert_eq!(points[8], (2, 2));
        assert_eq!(points[9], (1, 1));
        assert_eq!(points[10], (0, 0));
    }

    #[test]
    fn test_plot_line_south() {
        let mut points = Vec::new();
        plot_line(0, 10, 0, 0, |x, y| points.push((x, y)));
        assert_eq!(points.len(), 11);
        assert_eq!(points[0], (0, 10));
        assert_eq!(points[1], (0, 9));
        assert_eq!(points[2], (0, 8));
        assert_eq!(points[3], (0, 7));
        assert_eq!(points[4], (0, 6));
        assert_eq!(points[5], (0, 5));
        assert_eq!(points[6], (0, 4));
        assert_eq!(points[7], (0, 3));
        assert_eq!(points[8], (0, 2));
        assert_eq!(points[9], (0, 1));
        assert_eq!(points[10], (0, 0));
    }

    #[test]
    fn test_iterate_line() {
        #[derive(Debug)]
        struct TestCase {
            line: ULine,
            unit: IVec2,
        }

        let test_cases = vec![
            TestCase {
                line: ULine::new((10, 10), (10, 20)),
                unit: NORTH,
            },
            TestCase {
                line: ULine::new((10, 10), (20, 20)),
                unit: NORTH_EAST,
            },
            TestCase {
                line: ULine::new((10, 10), (20, 10)),
                unit: EAST,
            },
            TestCase {
                line: ULine::new((10, 10), (20, 0)),
                unit: SOUTH_EAST,
            },
            TestCase {
                line: ULine::new((10, 10), (10, 0)),
                unit: SOUTH,
            },
            TestCase {
                line: ULine::new((10, 10), (0, 0)),
                unit: SOUTH_WEST,
            },
            TestCase {
                line: ULine::new((10, 10), (0, 10)),
                unit: WEST,
            },
            TestCase {
                line: ULine::new((10, 10), (0, 20)),
                unit: NORTH_WEST,
            },
        ];

        for test_case in test_cases {
            let iters = &mut [
                LinePixelIterator::Axis(AxisLineIterator::new(&test_case.line).unwrap()),
                LinePixelIterator::Angle(AngleLineIterator::new(&test_case.line)),
            ];

            for iter in iters {
                let mut current = UVec2::splat(10);
                while let Some(p) = iter.peek() {
                    assert_eq!(p, current, "{:?}, Iter: {:?}", test_case, iter);
                    let n = iter.next().unwrap();
                    assert_eq!(n, current, "{:?}, Iter: {:?}", test_case, iter);

                    current = (current.as_ivec2() + test_case.unit).as_uvec2();
                }
                assert_eq!(iter.peek(), None, "{:?}, Iter: {:?}", test_case, iter);
                assert_eq!(iter.peek(), None, "{:?}, Iter: {:?}", test_case, iter);
                assert_eq!(iter.next(), None, "{:?}, Iter: {:?}", test_case, iter);
                assert_eq!(iter.next(), None, "{:?}, Iter: {:?}", test_case, iter);
            }
        }
    }

    #[test]
    fn test_seek_bounds() {
        #[derive(Debug)]
        struct TestCase {
            name: String,
            line: ULine,
            seek_bounds_ops: Vec<SeekBoundsOp>,
        }

        #[derive(Debug)]
        struct SeekBoundsOp {
            bounds: URect,
            expected_result: Option<UVec2>,
            expected_next: Option<UVec2>,
        }

        let test_cases = vec![
            TestCase {
                name: "N".to_string(),
                line: ULine::new((0, 0), (0, 10)),
                seek_bounds_ops: vec![
                    SeekBoundsOp {
                        bounds: URect::new(0, 0, 2, 2),
                        expected_result: Some((0, 1).into()),
                        expected_next: Some((0, 2).into()),
                    },
                    SeekBoundsOp {
                        bounds: URect::new(0, 2, 4, 6),
                        expected_result: Some((0, 5).into()),
                        expected_next: Some((0, 6).into()),
                    },
                    SeekBoundsOp {
                        bounds: URect::new(0, 6, 6, 12),
                        expected_result: Some((0, 10).into()),
                        expected_next: None,
                    },
                ],
            },
            TestCase {
                name: "E".to_string(),
                line: ULine::new((0, 0), (10, 0)),
                seek_bounds_ops: vec![
                    SeekBoundsOp {
                        bounds: URect::new(0, 0, 2, 2),
                        expected_result: Some((1, 0).into()),
                        expected_next: Some((2, 0).into()),
                    },
                    SeekBoundsOp {
                        bounds: URect::new(2, 0, 6, 4),
                        expected_result: Some((5, 0).into()),
                        expected_next: Some((6, 0).into()),
                    },
                    SeekBoundsOp {
                        bounds: URect::new(6, 0, 12, 6),
                        expected_result: Some((10, 0).into()),
                        expected_next: None,
                    },
                ],
            },
            TestCase {
                name: "S".to_string(),
                line: ULine::new((0, 20), (0, 10)),
                seek_bounds_ops: vec![
                    SeekBoundsOp {
                        bounds: URect::new(0, 18, 2, 20),
                        expected_result: Some((0, 18).into()),
                        expected_next: Some((0, 17).into()),
                    },
                    SeekBoundsOp {
                        bounds: URect::new(0, 14, 4, 18),
                        expected_result: Some((0, 14).into()),
                        expected_next: Some((0, 13).into()),
                    },
                    SeekBoundsOp {
                        bounds: URect::new(0, 8, 6, 14),
                        expected_result: Some((0, 10).into()),
                        expected_next: None,
                    },
                ],
            },
            TestCase {
                name: "W".to_string(),
                line: ULine::new((20, 0), (10, 0)),
                seek_bounds_ops: vec![
                    SeekBoundsOp {
                        bounds: URect::new(18, 0, 20, 2),
                        expected_result: Some((18, 0).into()),
                        expected_next: Some((17, 0).into()),
                    },
                    SeekBoundsOp {
                        bounds: URect::new(14, 0, 18, 4),
                        expected_result: Some((14, 0).into()),
                        expected_next: Some((13, 0).into()),
                    },
                    SeekBoundsOp {
                        bounds: URect::new(8, 0, 14, 6),
                        expected_result: Some((10, 0).into()),
                        expected_next: None,
                    },
                ],
            },
            TestCase {
                name: "NE".to_string(),
                line: ULine::new((0, 0), (10, 10)),
                seek_bounds_ops: vec![
                    SeekBoundsOp {
                        bounds: URect::new(0, 0, 2, 2),
                        expected_result: Some((1, 1).into()),
                        expected_next: Some((2, 2).into()),
                    },
                    SeekBoundsOp {
                        bounds: URect::new(2, 2, 6, 6),
                        expected_result: Some((5, 5).into()),
                        expected_next: Some((6, 6).into()),
                    },
                    SeekBoundsOp {
                        bounds: URect::new(6, 6, 12, 12),
                        expected_result: Some((10, 10).into()),
                        expected_next: None,
                    },
                ],
            },
            TestCase {
                name: "NW".to_string(),
                line: ULine::new((30, 20), (20, 30)),
                seek_bounds_ops: vec![
                    SeekBoundsOp {
                        bounds: URect::new(28, 20, 30, 22),
                        expected_result: Some((29, 21).into()),
                        expected_next: Some((28, 22).into()),
                    },
                    SeekBoundsOp {
                        bounds: URect::new(24, 22, 28, 26),
                        expected_result: Some((25, 25).into()),
                        expected_next: Some((24, 26).into()),
                    },
                    SeekBoundsOp {
                        bounds: URect::new(18, 26, 24, 32),
                        expected_result: Some((20, 30).into()),
                        expected_next: None,
                    },
                ],
            },
            TestCase {
                name: "SW".to_string(),
                line: ULine::new((20, 20), (10, 10)),
                seek_bounds_ops: vec![
                    SeekBoundsOp {
                        bounds: URect::new(18, 18, 20, 20),
                        expected_result: Some((18, 18).into()),
                        expected_next: Some((17, 17).into()),
                    },
                    SeekBoundsOp {
                        bounds: URect::new(14, 14, 18, 18),
                        expected_result: Some((14, 14).into()),
                        expected_next: Some((13, 13).into()),
                    },
                    SeekBoundsOp {
                        bounds: URect::new(8, 8, 14, 14),
                        expected_result: Some((10, 10).into()),
                        expected_next: None,
                    },
                ],
            },
            TestCase {
                name: "SE".to_string(),
                line: ULine::new((0, 20), (10, 10)),
                seek_bounds_ops: vec![
                    SeekBoundsOp {
                        bounds: URect::new(0, 18, 2, 20),
                        expected_result: Some((1, 19).into()),
                        expected_next: Some((2, 18).into()),
                    },
                    SeekBoundsOp {
                        bounds: URect::new(2, 14, 6, 18),
                        expected_result: Some((5, 15).into()),
                        expected_next: Some((6, 14).into()),
                    },
                    SeekBoundsOp {
                        bounds: URect::new(6, 8, 12, 14),
                        expected_result: Some((10, 10).into()),
                        expected_next: None,
                    },
                ],
            },
        ];

        for test_case in test_cases {
            let iters = &mut [
                LinePixelIterator::Axis(AxisLineIterator::new(&test_case.line).unwrap()),
                LinePixelIterator::Angle(AngleLineIterator::new(&test_case.line)),
            ];

            for iter in iters {
                for op in &test_case.seek_bounds_ops {
                    let p = iter.seek_bounds(&op.bounds);
                    assert_eq!(
                        p, op.expected_result,
                        "{}: Iter: {:?} Result: Line: {:?}, op: {:?}",
                        &test_case.name, &iter, &test_case.line, op
                    );
                    assert_eq!(
                        iter.next(),
                        op.expected_next,
                        "{}: Iter: {:?} Next: Line: {:?}, op: {:?}",
                        &test_case.name,
                        &iter,
                        &test_case.line,
                        op
                    );
                }
            }
        }
    }

    #[test]
    fn test_angle_line_iterator() {
        let test_cases = vec![
            (10, 20, 10, 10),
            (20, 10, 10, 10),
            (10, 10, 20, 10),
            (20, 20, 10, 10),
            (10, 20, 10, 10),
            (15, 15, 30, 20),
            (20, 15, 15, 30),
            (10, 10, 10, 10),
            (10, 10, 20, 20),
            (10, 10, 0, 20),
            (10, 10, 0, 0),
            (10, 10, 20, 0),
        ];
        for test_case in test_cases {
            let line = ULine::new(
                uvec2(test_case.0, test_case.1),
                uvec2(test_case.2, test_case.3),
            );
            let mut it = AngleLineIterator::new(&line);
            plot_line(
                test_case.0 as i32,
                test_case.1 as i32,
                test_case.2 as i32,
                test_case.3 as i32,
                |x, y| assert_eq!(it.next(), Some((x as u32, y as u32).into())),
            );
            assert_eq!(it.next(), None);
            assert_eq!(it.next(), None);
        }
    }
}
