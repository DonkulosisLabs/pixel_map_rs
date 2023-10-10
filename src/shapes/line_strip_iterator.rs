use crate::{iline, ILine, LinePixelIterator};
use bevy_math::IVec2;

/// An iterator that yields all pixel coordinates in a line strip.
/// A given coordinate in the line strip will only be yielded once, assuming line segments
/// are unique, regardless of the line strip being open or closed,
/// or whether the line segments are contiguous or not.
pub struct LineStripPixelIterator {
    line_iters: Vec<LinePixelIterator>,
    last_seen: IVec2,
    closed: bool,
}

impl LineStripPixelIterator {
    /// Create a new iterator from a slice of points. A line is drawn between each point and the next.
    /// For example, if points `A`, `B`, and `C` are provided, the following line segments will
    /// be iterated over: `AB`, `BC`.
    pub fn from_points(points: &[IVec2]) -> Self {
        let lines: Vec<ILine> = points.windows(2).map(|w| iline(w[0], w[1])).collect();
        Self::from_lines(&lines)
    }

    /// Create a new iterator from a slice of lines. Each line will be iterated over to yield pixels.
    pub fn from_lines(lines: &[ILine]) -> Self {
        let closed = lines
            .first()
            .map(|first| first.start() == lines.last().unwrap().end())
            .unwrap_or(false);
        let line_iters: Vec<LinePixelIterator> = lines.iter().map(LinePixelIterator::new).collect();
        Self {
            line_iters,
            last_seen: IVec2::ZERO,
            closed,
        }
    }
}

impl Iterator for LineStripPixelIterator {
    type Item = IVec2;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.line_iters.is_empty() {
            return None;
        }

        let iter = &mut self.line_iters[0];
        match iter.next() {
            Some(point) => {
                // Is this the last point in a closed line strip?
                if self.closed && self.line_iters.len() == 1 && self.line_iters[0].peek().is_none()
                {
                    // Skip it, because we already yielded it when we started.
                    None
                } else {
                    self.last_seen = point;
                    Some(point)
                }
            }
            None => {
                // We're at the end of the line segment, so remove it from the list.
                self.line_iters.remove(0);

                // Is there another line segment to iterate over?
                if let Some(line_iter) = self.line_iters.first() {
                    if let Some(next) = line_iter.peek() {
                        if next == self.last_seen {
                            // Handle contiguous line strip:
                            // Skip the first point, which was already returned by the previous iterator.
                            self.line_iters[0].next();
                        }
                    }
                }
                self.next()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_math::ivec2;

    #[test]
    fn test_open_line_strip() {
        use super::*;
        let points = vec![ivec2(0, 0), ivec2(1, 0), ivec2(3, 0)];
        let mut iter = LineStripPixelIterator::from_points(&points);
        assert_eq!(iter.next(), Some(ivec2(0, 0)));
        assert_eq!(iter.next(), Some(ivec2(1, 0)));
        assert_eq!(iter.next(), Some(ivec2(2, 0)));
        assert_eq!(iter.next(), Some(ivec2(3, 0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_closed_line_strip() {
        let points = vec![ivec2(0, 0), ivec2(1, 0), ivec2(2, 0), ivec2(0, 0)];
        let mut iter = LineStripPixelIterator::from_points(&points);
        assert_eq!(iter.next(), Some(ivec2(0, 0)));
        assert_eq!(iter.next(), Some(ivec2(1, 0)));
        assert_eq!(iter.next(), Some(ivec2(2, 0)));
        assert_eq!(iter.next(), Some(ivec2(1, 0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_non_contiguous_open_line_strip() {
        let lines = vec![iline((0, 0), (1, 0)), iline((3, 1), (4, 1))];
        let mut iter = LineStripPixelIterator::from_lines(&lines);
        assert_eq!(iter.next(), Some(ivec2(0, 0)));
        assert_eq!(iter.next(), Some(ivec2(1, 0)));
        assert_eq!(iter.next(), Some(ivec2(3, 1)));
        assert_eq!(iter.next(), Some(ivec2(4, 1)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_non_contiguous_closed_line_strip() {
        let lines = vec![
            iline((0, 0), (1, 0)),
            iline((3, 1), (4, 1)),
            iline((0, 1), (0, 0)),
        ];
        let mut iter = LineStripPixelIterator::from_lines(&lines);
        assert_eq!(iter.next(), Some(ivec2(0, 0)));
        assert_eq!(iter.next(), Some(ivec2(1, 0)));
        assert_eq!(iter.next(), Some(ivec2(3, 1)));
        assert_eq!(iter.next(), Some(ivec2(4, 1)));
        assert_eq!(iter.next(), Some(ivec2(0, 1)));
        assert_eq!(iter.next(), None);
    }
}
