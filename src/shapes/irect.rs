#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::ILine;
use bevy_math::{IRect, IVec2};

/// Get the four points that make up the corners of the given `rect`.
#[inline]
#[must_use]
pub fn rect_points(rect: &IRect) -> [IVec2; 4] {
    [
        rect.min,
        rect.min + IVec2::new(rect.width(), 0),
        rect.max,
        rect.min + IVec2::new(0, rect.height()),
    ]
}

/// Get the four lines that make up the edges of this rectangle.
#[inline]
#[must_use]
pub fn rect_segments(rect: &IRect) -> [ILine; 4] {
    let width = rect.max.x - rect.min.x;
    let height = rect.max.y - rect.min.y;
    [
        ILine::new(rect.min, rect.min + IVec2::new(width, 0)),
        ILine::new(rect.min + IVec2::new(width, 0), rect.max),
        ILine::new(rect.max, rect.min + IVec2::new(0, height)),
        ILine::new(rect.min + IVec2::new(0, height), rect.min),
    ]
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
        let x = rect.min.x;
        let y = rect.min.y;
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
    fn test_pixel_iterator() {
        let rect = IRect::new(1, 1, 3, 3);
        let mut iter = RectPixelIterator::new(rect);
        assert_eq!(iter.next(), Some((1, 1).into()));
        assert_eq!(iter.next(), Some((2, 1).into()));
        assert_eq!(iter.next(), Some((1, 2).into()));
        assert_eq!(iter.next(), Some((2, 2).into()));
        assert_eq!(iter.next(), None);
    }
}
