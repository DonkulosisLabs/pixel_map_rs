#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::to_cropped_urect;
use bevy_math::{ivec2, IRect, IVec2, URect, UVec2};

/// A circle represented by a center point, in integer coordinates, and a radius.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct ICircle {
    point: IVec2,
    radius: u32,
}

impl ICircle {
    pub const ZERO: Self = Self {
        point: IVec2::ZERO,
        radius: 0,
    };

    /// Creates a new circle with the given center point and radius.
    #[inline]
    #[must_use]
    pub fn new<P>(point: P, radius: u32) -> Self
    where
        P: Into<IVec2>,
    {
        Self {
            point: point.into(),
            radius,
        }
    }

    /// Get the center point `x` component.
    #[inline]
    #[must_use]
    pub fn x(&self) -> i32 {
        self.point.x
    }

    /// Get the center point `y` component.
    #[inline]
    #[must_use]
    pub fn y(&self) -> i32 {
        self.point.y
    }

    /// Get the center point.
    #[inline]
    #[must_use]
    pub fn point(&self) -> IVec2 {
        self.point
    }

    /// Get the radius.
    #[inline]
    #[must_use]
    pub fn radius(&self) -> u32 {
        self.radius
    }

    /// Determine if the circle contains the given point.
    #[inline]
    #[must_use]
    pub fn contains<P>(&self, point: P) -> bool
    where
        P: Into<IVec2>,
    {
        let d = point.into() - self.point;
        d.x * d.x + d.y * d.y <= self.radius as i32 * self.radius as i32
    }

    /// Get the axis-aligned bounding box of the circle.
    #[inline]
    #[must_use]
    pub fn aabb(&self) -> URect {
        let size = self.radius * 2;
        let irect = IRect::from_center_size(self.point, IVec2::splat(size as i32));
        to_cropped_urect(&irect)
    }

    /// Get the axis-aligned largest rectangle contained within the circle.
    #[inline]
    #[must_use]
    pub fn inner_rect(&self) -> URect {
        let size = (self.radius as f32 * 2f32.sqrt()) as u32;
        let irect = IRect::from_center_size(self.point, IVec2::splat(size as i32));
        to_cropped_urect(&irect)
    }

    /// Iterator over pixels in the circle.
    #[inline]
    #[must_use]
    pub fn pixels(&self) -> ICirclePixelIterator {
        ICirclePixelIterator::new(self.clone())
    }

    /// Iterator over pixels in the circle, cropped to unsigned space.
    #[inline]
    #[must_use]
    pub fn cropped_pixels(&self) -> CroppedCirclePixelIterator {
        CroppedCirclePixelIterator::from_circle(self)
    }
}

impl From<IRect> for ICircle {
    #[inline]
    fn from(rect: IRect) -> Self {
        ICircle::from(&rect)
    }
}
impl From<&IRect> for ICircle {
    #[inline]
    fn from(rect: &IRect) -> Self {
        let radius = (rect.width().min(rect.height()) / 2) as u32;
        let p = (
            rect.min.x + rect.width() / 2,
            rect.min.y + rect.height() / 2,
        );
        Self::new(p, radius)
    }
}

pub struct CroppedCirclePixelIterator {
    inner: ICirclePixelIterator,
}

impl CroppedCirclePixelIterator {
    #[inline]
    #[must_use]
    pub fn from_circle(circle: &ICircle) -> Self {
        Self {
            inner: circle.pixels(),
        }
    }
}

impl Iterator for CroppedCirclePixelIterator {
    type Item = UVec2;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let i = self.inner.next()?;
            if i.x < 0 || i.y < 0 {
                continue;
            } else {
                return Some(i.as_uvec2());
            }
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ICirclePixelIterator {
    circle: ICircle,
    x: i32,
    y: i32,
}

impl ICirclePixelIterator {
    #[inline]
    #[must_use]
    pub fn new(circle: ICircle) -> Self {
        let y = -(circle.radius as i32);
        let r = circle.radius as i32;
        let x = -((r * r - y * y) as f32).sqrt() as i32;
        Self { circle, x, y }
    }
}

impl Iterator for ICirclePixelIterator {
    type Item = IVec2;

    fn next(&mut self) -> Option<Self::Item> {
        let r = self.circle.radius as i32;
        if self.y > r {
            return None;
        }

        let x = self.x;
        self.x += 1;

        let x_len = ((r * r - self.y * self.y) as f32).sqrt() as i32;
        if x > x_len {
            self.y += 1;
            let x_len = ((r * r - self.y * self.y) as f32).sqrt() as i32;
            self.x = -x_len;
            self.next()
        } else {
            let x = self.circle.x() + x;
            let y = self.circle.y() + self.y;
            Some(ivec2(x, y))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_rect() {
        let rect = IRect::new(0, 0, 10, 10);
        let circle = ICircle::from(rect);
        assert_eq!(circle.x(), 5);
        assert_eq!(circle.y(), 5);
        assert_eq!(circle.radius(), 5);
    }

    #[test]
    fn test_pixels() {
        let mut iter = ICircle::new((0, 0), 2).pixels();
        assert_eq!(iter.next(), Some((0, -2).into()));
        assert_eq!(iter.next(), Some((-1, -1).into()));
        assert_eq!(iter.next(), Some((0, -1).into()));
        assert_eq!(iter.next(), Some((1, -1).into()));
        assert_eq!(iter.next(), Some((-2, 0).into()));
        assert_eq!(iter.next(), Some((-1, 0).into()));
        assert_eq!(iter.next(), Some((0, 0).into()));
        assert_eq!(iter.next(), Some((1, 0).into()));
        assert_eq!(iter.next(), Some((2, 0).into()));
        assert_eq!(iter.next(), Some((-1, 1).into()));
        assert_eq!(iter.next(), Some((0, 1).into()));
        assert_eq!(iter.next(), Some((1, 1).into()));
        assert_eq!(iter.next(), Some((0, 2).into()));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }
}