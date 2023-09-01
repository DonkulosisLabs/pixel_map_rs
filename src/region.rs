#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::quadrant::Quadrant;
use bevy_math::{IRect, IVec2};
use num_traits::{NumCast, Unsigned};

/// A square region defined by a bottom-left point and a size, in integer units.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Region<U: Unsigned + Copy = u16> {
    x: U,
    y: U,
    size: U,
}

impl<U: Unsigned + NumCast + Copy> Region<U> {
    /// Create a new region with the given bottom-left point coordinates and size.
    #[inline]
    #[must_use]
    pub fn new(x: U, y: U, size: U) -> Self {
        Self { x, y, size }
    }

    /// Get the `x` component of the bottom-left point.
    #[inline]
    #[must_use]
    pub fn x(&self) -> U {
        self.x
    }

    /// Get the `y` component of the bottom-left point.
    #[inline]
    #[must_use]
    pub fn y(&self) -> U {
        self.y
    }

    /// Get the bottom-left point.
    #[inline]
    #[must_use]
    pub fn point(&self) -> IVec2 {
        let x = num_traits::cast::cast(self.x).unwrap();
        let y = num_traits::cast::cast(self.y).unwrap();
        (x, y).into()
    }

    /// Get the top-right point.
    #[inline]
    #[must_use]
    pub fn end_point(&self) -> IVec2 {
        let size: i32 = num_traits::cast::cast(self.size).unwrap();
        self.point() + size
    }

    /// Get the size of the region.
    #[inline]
    #[must_use]
    pub fn size(&self) -> U {
        self.size
    }

    #[inline]
    #[must_use]
    pub fn size_as<N: NumCast>(&self) -> N {
        num_traits::cast::cast::<U, N>(self.size).unwrap()
    }

    /// Get the center point of the region.
    #[inline]
    #[must_use]
    pub fn center(&self) -> U {
        self.size / U::from(2).unwrap()
    }

    /// Determine if this region represents the smallest possible unit or pixel size.
    #[inline]
    #[must_use]
    pub fn is_unit(&self, pixel_size: u8) -> bool {
        self.size == num_traits::cast(pixel_size).unwrap()
    }

    /// Determine if the given point is contained within this region.
    #[inline]
    #[must_use]
    pub fn contains<P>(&self, point: P) -> bool
    where
        P: Into<IVec2>,
    {
        let point = point.into();
        let x: i32 = match num_traits::cast(self.x) {
            Some(x) => x,
            None => return false,
        };
        let y: i32 = match num_traits::cast(self.y) {
            Some(y) => y,
            None => return false,
        };
        let size: i32 = match num_traits::cast(self.size) {
            Some(size) => size,
            None => return false,
        };
        point.x >= x && point.x < x + size && point.y >= y && point.y < y + size
    }

    /// Obtain the quadrant for the given point in relation to the center of this region.
    #[inline]
    #[must_use]
    pub fn quadrant_for<P>(&self, point: P) -> Quadrant
    where
        P: Into<IVec2>,
    {
        let point = point.into();
        let center = num_traits::cast(self.center()).unwrap();
        Quadrant::for_point(point - self.point(), center)
    }
}

impl<U: Unsigned + NumCast + Copy> Into<IRect> for Region<U> {
    #[inline]
    #[must_use]
    fn into(self) -> IRect {
        IRect::from_corners(self.point(), self.end_point())
    }
}

impl<U: Unsigned + NumCast + Copy> Into<IRect> for &Region<U> {
    #[inline]
    #[must_use]
    fn into(self) -> IRect {
        IRect::from_corners(self.point(), self.end_point())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_contains() {
        let r = Region::new(0u32, 0, 4);
        assert!(r.contains((0, 0)));
        assert!(r.contains((1, 0)));
        assert!(r.contains((2, 0)));
        assert!(r.contains((3, 0)));
        assert!(r.contains((0, 1)));
        assert!(r.contains((1, 1)));
        assert!(r.contains((2, 1)));
        assert!(r.contains((3, 1)));
        assert!(r.contains((0, 2)));
        assert!(r.contains((1, 2)));
        assert!(r.contains((2, 2)));
        assert!(r.contains((3, 2)));
        assert!(r.contains((0, 3)));
        assert!(r.contains((1, 3)));
        assert!(r.contains((2, 3)));
        assert!(r.contains((3, 3)));
        assert!(!r.contains((4, 0)));
        assert!(!r.contains((0, 4)));
    }

    #[test]
    fn test_quadrant_for() {
        let r = Region::new(0u32, 0, 4);
        assert_eq!(r.quadrant_for((0, 0)), Quadrant::BottomLeft);
        assert_eq!(r.quadrant_for((1, 0)), Quadrant::BottomLeft);
        assert_eq!(r.quadrant_for((2, 0)), Quadrant::BottomRight);
        assert_eq!(r.quadrant_for((3, 0)), Quadrant::BottomRight);
        assert_eq!(r.quadrant_for((0, 1)), Quadrant::BottomLeft);
        assert_eq!(r.quadrant_for((1, 1)), Quadrant::BottomLeft);
        assert_eq!(r.quadrant_for((2, 1)), Quadrant::BottomRight);
        assert_eq!(r.quadrant_for((3, 1)), Quadrant::BottomRight);
        assert_eq!(r.quadrant_for((0, 2)), Quadrant::TopLeft);
        assert_eq!(r.quadrant_for((1, 2)), Quadrant::TopLeft);
        assert_eq!(r.quadrant_for((2, 2)), Quadrant::TopRight);
        assert_eq!(r.quadrant_for((3, 2)), Quadrant::TopRight);
        assert_eq!(r.quadrant_for((0, 3)), Quadrant::TopLeft);
        assert_eq!(r.quadrant_for((1, 3)), Quadrant::TopLeft);
        assert_eq!(r.quadrant_for((2, 3)), Quadrant::TopRight);
        assert_eq!(r.quadrant_for((3, 3)), Quadrant::TopRight);
    }
}
