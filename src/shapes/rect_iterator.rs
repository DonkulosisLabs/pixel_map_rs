use bevy_math::{uvec2, URect, UVec2};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Iterate all pixel coordinates in a [URect].
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct URectPixelIterator {
    rect: URect,
    x: u32,
    y: u32,
}

impl URectPixelIterator {
    #[inline]
    #[must_use]
    pub fn new(rect: URect) -> Self {
        let x = rect.min.x;
        let y = rect.min.y;
        Self { rect, x, y }
    }
}

impl Iterator for URectPixelIterator {
    type Item = UVec2;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.x < self.rect.max.x {
            let x = self.x;
            self.x += 1;
            Some(uvec2(x, self.y))
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
        let rect = URect::new(1, 1, 3, 3);
        let mut iter = URectPixelIterator::new(rect);
        assert_eq!(iter.next(), Some((1, 1).into()));
        assert_eq!(iter.next(), Some((2, 1).into()));
        assert_eq!(iter.next(), Some((1, 2).into()));
        assert_eq!(iter.next(), Some((2, 2).into()));
        assert_eq!(iter.next(), None);
    }
}
