use super::{Direction, IRect, Line};
use glam::IVec2;

pub(super) fn plot_line<F>(x0: i32, y0: i32, x1: i32, y1: i32, mut plot: F)
where
    F: FnMut(i32, i32),
{
    let dx = i32::abs(x1 - x0);
    let dy = i32::abs(y1 - y0);
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

#[derive(Debug, Clone, PartialEq)]
pub enum LineIterator {
    Axis(AxisLineIterator),
    Angle(AngleLineIterator),
}

impl LineIterator {
    #[inline]
    pub(super) fn new(line: &Line) -> Self {
        match AxisLineIterator::new(line) {
            Some(iter) => LineIterator::Axis(iter),
            None => LineIterator::Angle(AngleLineIterator::new(line)),
        }
    }

    #[inline]
    pub fn peek(&self) -> Option<IVec2> {
        match self {
            LineIterator::Axis(iter) => iter.peek(),
            LineIterator::Angle(iter) => iter.peek(),
        }
    }

    /// Seek the iterator to the first point on the line that is on the given bounds, and return it.
    /// Calling next() after this will return the point beyond the bounds, if there is one
    /// for the line segment.
    /// Returns None if the end of the iterator is reached without touching the bounds.
    #[inline]
    pub fn seek_bounds(&mut self, bounds: &IRect) -> Option<IVec2> {
        match self {
            LineIterator::Axis(iter) => iter.seek_bounds(bounds),
            LineIterator::Angle(iter) => iter.seek_bounds(bounds),
        }
    }
}

impl Iterator for LineIterator {
    type Item = IVec2;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            LineIterator::Axis(iter) => iter.next(),
            LineIterator::Angle(iter) => iter.next(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AxisLineIterator {
    point: IVec2,
    direction: Direction,
    end: IVec2,
    finished: bool,
}

impl AxisLineIterator {
    #[inline]
    pub(super) fn new(line: &Line) -> Option<Self> {
        let direction = line.axis_alignment().or(line.diagonal_axis_alignment())?;
        Some(Self {
            point: line.start(),
            direction,
            end: line.end(),
            finished: false,
        })
    }

    #[inline]
    pub fn peek(&self) -> Option<IVec2> {
        if self.finished {
            return None;
        }
        Some(self.point)
    }

    pub fn seek_bounds(&mut self, bounds: &IRect) -> Option<IVec2> {
        let point = self.next()?;

        let top = bounds.top_bounds();
        let left = bounds.left_bounds();
        let right = bounds.right_bounds();
        let bottom = bounds.bottom_bounds();

        let result = match self.direction {
            Direction::North => Some(IVec2::new(point.x, self.end.y.min(top))),
            Direction::NorthEast => {
                let x_distance = right - self.end.x;
                let y_distance = top - self.end.y;
                let (x, y) = if x_distance < y_distance {
                    (right, self.end.y + x_distance)
                } else {
                    (self.end.x + y_distance, top)
                };
                if y > point.y {
                    Some(IVec2::new(x, y))
                } else {
                    Some(point)
                }
            }
            Direction::NorthWest => {
                let x_distance = self.end.x - left;
                let y_distance = top - self.end.y;
                let (x, y) = if x_distance < y_distance {
                    (left, self.end.y + x_distance)
                } else {
                    (self.end.x - y_distance, top)
                };
                if y > point.y {
                    Some(IVec2::new(x, y))
                } else {
                    Some(point)
                }
            }
            Direction::East => Some(IVec2::new(self.end.x.min(right), point.y)),
            Direction::South => Some(IVec2::new(point.x, self.end.y.max(bottom))),
            Direction::SouthEast => {
                let x_distance = right - self.end.x;
                let y_distance = self.end.y - bottom;
                let (x, y) = if x_distance < y_distance {
                    (right, self.end.y - x_distance)
                } else {
                    (self.end.x + y_distance, bottom)
                };
                if y < point.y {
                    Some(IVec2::new(x, y))
                } else {
                    Some(point)
                }
            }
            Direction::SouthWest => {
                let x_distance = self.end.x - left;
                let y_distance = self.end.y - bottom;
                let (x, y) = if x_distance < y_distance {
                    (left, self.end.y - x_distance)
                } else {
                    (self.end.x - y_distance, bottom)
                };
                if y < point.y {
                    Some(IVec2::new(x, y))
                } else {
                    Some(point)
                }
            }
            Direction::West => Some(IVec2::new(self.end.x.max(left), point.y)),
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
    type Item = IVec2;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            None
        } else {
            let result = self.point;
            if self.point == self.end {
                self.finished = true;
            } else {
                self.point = self.point + self.direction.unit();
            }
            Some(result)
        }
    }
}

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
    pub(super) fn new(line: &Line) -> Self {
        let x0 = line.start().x;
        let y0 = line.start().y;
        let x1 = line.end().x;
        let y1 = line.end().y;
        let dist = IVec2::new(i32::abs(x1 - x0), i32::abs(y1 - y0));
        let xi = if x1 < x0 { -1 } else { 1 };
        let yi = if y1 < y0 { -1 } else { 1 };
        AngleLineIterator {
            end: line.end(),
            dist,
            point: line.start(),
            xi,
            yi,
            err: dist.x - dist.y,
            e2: 0,
            finished: false,
        }
    }

    #[inline]
    pub fn peek(&self) -> Option<IVec2> {
        if self.finished {
            return None;
        }
        Some(self.point)
    }

    #[inline]
    pub fn seek_bounds(&mut self, bounds: &IRect) -> Option<IVec2> {
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
    type Item = IVec2;

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
                    self.point += IVec2::new(self.xi, 0);
                }
                if self.e2 < self.dist.x {
                    self.err += self.dist.x;
                    self.point += IVec2::new(0, self.yi);
                }
            }
            Some(result)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
            line: Line,
            unit: IVec2,
        }

        let test_cases = vec![
            TestCase {
                line: Line::new((0, 0), (0, 10)), // N
                unit: (0, 1).into(),
            },
            TestCase {
                line: Line::new((0, 0), (10, 10)), // NE
                unit: (1, 1).into(),
            },
            TestCase {
                line: Line::new((0, 0), (10, 0)), // E
                unit: (1, 0).into(),
            },
            TestCase {
                line: Line::new((0, 0), (10, -10)), // SE
                unit: (1, -1).into(),
            },
            TestCase {
                line: Line::new((0, 0), (0, -10)), // S
                unit: (0, -1).into(),
            },
            TestCase {
                line: Line::new((0, 0), (-10, -10)), // SW
                unit: (-1, -1).into(),
            },
            TestCase {
                line: Line::new((0, 0), (-10, 0)), // W
                unit: (-1, 0).into(),
            },
            TestCase {
                line: Line::new((0, 0), (-10, 10)), // NW
                unit: (-1, 1).into(),
            },
        ];

        for test_case in test_cases {
            let iters = &mut [
                LineIterator::Axis(AxisLineIterator::new(&test_case.line).unwrap()),
                LineIterator::Angle(AngleLineIterator::new(&test_case.line)),
            ];

            for iter in iters {
                let mut current = IVec2::default();
                while let Some(p) = iter.peek() {
                    assert_eq!(p, current, "{:?}, Iter: {:?}", test_case, iter);
                    let n = iter.next().unwrap();
                    assert_eq!(n, current, "{:?}, Iter: {:?}", test_case, iter);

                    current = current + test_case.unit;
                }
                assert_eq!(iter.peek(), None, "{:?}, Iter: {:?}", test_case, iter);
                assert_eq!(iter.peek(), None, "{:?}, Iter: {:?}", test_case, iter);
                assert_eq!(iter.next(), None, "{:?}, Iter: {:?}", test_case, iter);
                assert_eq!(iter.next(), None, "{:?}, Iter: {:?}", test_case, iter);
            }
        }
    }

    //#[test]
    fn test_seek_bounds() {
        #[derive(Debug)]
        struct TestCase {
            line: Line,
            seek_bounds_ops: Vec<SeekBoundsOp>,
        }

        #[derive(Debug)]
        struct SeekBoundsOp {
            bounds: IRect,
            expected_result: Option<IVec2>,
            expected_next: Option<IVec2>,
        }

        let test_cases = vec![
            TestCase {
                line: Line::new((0, 0), (0, 10)), // N
                seek_bounds_ops: vec![
                    SeekBoundsOp {
                        bounds: IRect::new(0, 0, 2, 2),
                        expected_result: Some((0, 1).into()),
                        expected_next: Some((0, 2).into()),
                    },
                    SeekBoundsOp {
                        bounds: IRect::new(0, 2, 4, 6),
                        expected_result: Some((0, 5).into()),
                        expected_next: Some((0, 6).into()),
                    },
                    SeekBoundsOp {
                        bounds: IRect::new(0, 6, 6, 12),
                        expected_result: Some((0, 10).into()),
                        expected_next: None,
                    },
                ],
            },
            TestCase {
                line: Line::new((0, 0), (10, 10)), // NE
                seek_bounds_ops: vec![
                    SeekBoundsOp {
                        bounds: IRect::new(0, 0, 2, 2),
                        expected_result: Some((1, 1).into()),
                        expected_next: Some((2, 2).into()),
                    },
                    SeekBoundsOp {
                        bounds: IRect::new(2, 2, 6, 6),
                        expected_result: Some((5, 5).into()),
                        expected_next: Some((6, 6).into()),
                    },
                    SeekBoundsOp {
                        bounds: IRect::new(6, 6, 12, 12),
                        expected_result: Some((10, 10).into()),
                        expected_next: None,
                    },
                ],
            },
            TestCase {
                line: Line::new((10, 0), (0, 10)), // NW
                seek_bounds_ops: vec![
                    SeekBoundsOp {
                        bounds: IRect::new(8, 0, 10, 2),
                        expected_result: Some((8, 2).into()),
                        expected_next: Some((7, 3).into()),
                    },
                    SeekBoundsOp {
                        bounds: IRect::new(4, 2, 8, 6),
                        expected_result: Some((4, 6).into()),
                        expected_next: Some((3, 7).into()),
                    },
                    SeekBoundsOp {
                        bounds: IRect::new(-2, 6, 4, 12),
                        expected_result: Some((10, 10).into()),
                        expected_next: None,
                    },
                ],
            },
            TestCase {
                line: Line::new((0, 0), (10, 0)), // E
                seek_bounds_ops: vec![
                    SeekBoundsOp {
                        bounds: IRect::new(0, 0, 2, 2),
                        expected_result: Some((1, 0).into()),
                        expected_next: Some((2, 0).into()),
                    },
                    SeekBoundsOp {
                        bounds: IRect::new(2, 0, 6, 4),
                        expected_result: Some((5, 0).into()),
                        expected_next: Some((6, 0).into()),
                    },
                    SeekBoundsOp {
                        bounds: IRect::new(6, 0, 12, 6),
                        expected_result: Some((10, 0).into()),
                        expected_next: None,
                    },
                ],
            },
            TestCase {
                line: Line::new((0, 0), (0, -10)), // S
                seek_bounds_ops: vec![
                    SeekBoundsOp {
                        bounds: IRect::new(0, -2, 2, 0),
                        expected_result: Some((0, -1).into()),
                        expected_next: Some((0, -2).into()),
                    },
                    SeekBoundsOp {
                        bounds: IRect::new(0, -6, 4, -2),
                        expected_result: Some((0, -5).into()),
                        expected_next: Some((0, -6).into()),
                    },
                    SeekBoundsOp {
                        bounds: IRect::new(0, -12, 6, -6),
                        expected_result: Some((0, -10).into()),
                        expected_next: None,
                    },
                ],
            },
            TestCase {
                line: Line::new((0, 0), (-10, 0)), // W
                seek_bounds_ops: vec![
                    SeekBoundsOp {
                        bounds: IRect::new(-2, 0, 0, 2),
                        expected_result: Some((-1, 0).into()),
                        expected_next: Some((-2, 0).into()),
                    },
                    SeekBoundsOp {
                        bounds: IRect::new(-6, 0, -2, 4),
                        expected_result: Some((-5, 0).into()),
                        expected_next: Some((-6, 0).into()),
                    },
                    SeekBoundsOp {
                        bounds: IRect::new(-12, 0, -6, 6),
                        expected_result: Some((-10, 0).into()),
                        expected_next: None,
                    },
                ],
            },
            TestCase {
                line: Line::new((0, 0), (-10, -10)), // SW
                seek_bounds_ops: vec![
                    SeekBoundsOp {
                        bounds: IRect::new(-2, -2, 0, 0),
                        expected_result: Some((-1, -1).into()),
                        expected_next: Some((-2, -2).into()),
                    },
                    SeekBoundsOp {
                        bounds: IRect::new(-6, -6, -2, -2),
                        expected_result: Some((-5, -5).into()),
                        expected_next: Some((-6, -6).into()),
                    },
                    SeekBoundsOp {
                        bounds: IRect::new(-12, -12, -6, -6),
                        expected_result: Some((-10, -10).into()),
                        expected_next: None,
                    },
                ],
            },
            TestCase {
                line: Line::new((0, 0), (10, -10)), // SE
                seek_bounds_ops: vec![
                    SeekBoundsOp {
                        bounds: IRect::new(0, -2, 2, 0),
                        expected_result: Some((1, -1).into()),
                        expected_next: Some((2, -2).into()),
                    },
                    SeekBoundsOp {
                        bounds: IRect::new(2, -6, 6, -2),
                        expected_result: Some((5, -5).into()),
                        expected_next: Some((6, -6).into()),
                    },
                    SeekBoundsOp {
                        bounds: IRect::new(6, -12, 12, -6),
                        expected_result: Some((10, -10).into()),
                        expected_next: None,
                    },
                ],
            },
        ];

        for test_case in test_cases {
            let iters = &mut [
                LineIterator::Axis(AxisLineIterator::new(&test_case.line).unwrap()),
                LineIterator::Angle(AngleLineIterator::new(&test_case.line)),
            ];

            for iter in iters {
                for op in &test_case.seek_bounds_ops {
                    let p = iter.seek_bounds(&op.bounds);
                    assert_eq!(
                        p, op.expected_result,
                        "Line: {:?}, op: {:?}",
                        &test_case.line, op
                    );
                    assert_eq!(
                        iter.next(),
                        op.expected_next,
                        "Line: {:?}, op: {:?}",
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
            (0, 10, 0, 0),
            (10, 0, 0, 0),
            (0, 0, 10, 0),
            (10, 10, 0, 0),
            (0, 10, 0, 0),
            (5, 5, 20, 10),
            (10, 5, 5, 20),
            (0, 0, 0, 0),
            (0, 0, 10, 10),
            (0, 0, -10, 10),
            (0, 0, -10, -10),
            (0, 0, 10, -10),
        ];
        for test_case in test_cases {
            let line = Line::new(
                IVec2::new(test_case.0, test_case.1),
                IVec2::new(test_case.2, test_case.3),
            );
            let mut it = AngleLineIterator::new(&line);
            plot_line(
                test_case.0,
                test_case.1,
                test_case.2,
                test_case.3,
                |x, y| assert_eq!(it.next(), Some((x, y).into())),
            );
            assert_eq!(it.next(), None);
            assert_eq!(it.next(), None);
        }
    }
}
