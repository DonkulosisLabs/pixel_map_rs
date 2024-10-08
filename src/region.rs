#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

use super::quadrant::Quadrant;
use bevy_math::{IRect, IVec2, URect, UVec2};
use num_traits::{NumCast, Unsigned};

/// A square region defined by a bottom-left point and a size, in integer units.
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
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
    pub fn point(&self) -> UVec2 {
        let x = num_traits::cast::cast(self.x).unwrap();
        let y = num_traits::cast::cast(self.y).unwrap();
        (x, y).into()
    }

    /// Get the top-right point.
    #[inline]
    #[must_use]
    pub fn end_point(&self) -> UVec2 {
        let size: u32 = num_traits::cast::cast(self.size).unwrap();
        self.point() + size
    }

    /// Get the center point.
    #[inline]
    #[must_use]
    pub fn center(&self) -> UVec2 {
        let size: u32 = self.size_as();
        self.point() + (size / 2)
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

    /// Get the region size / 2.
    #[inline]
    #[must_use]
    pub fn half_size(&self) -> U {
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
    pub fn contains_upoint<P>(&self, point: P) -> bool
    where
        P: Into<UVec2>,
    {
        let point = point.into();
        let x: u32 = match num_traits::cast(self.x) {
            Some(x) => x,
            None => return false,
        };
        let y: u32 = match num_traits::cast(self.y) {
            Some(y) => y,
            None => return false,
        };
        let size: u32 = match num_traits::cast(self.size) {
            Some(size) => size,
            None => return false,
        };
        point.x >= x && point.x < x + size && point.y >= y && point.y < y + size
    }

    /// Determine if the given point is contained within this region.
    #[inline]
    #[must_use]
    pub fn contains_ipoint<P>(&self, point: P) -> bool
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
    pub fn quadrant_for_upoint<P>(&self, point: P) -> Quadrant
    where
        P: Into<UVec2>,
    {
        let point = point.into();
        let center = num_traits::cast(self.half_size()).unwrap();
        Quadrant::for_upoint(point - self.point(), center)
    }

    /// Obtain the quadrant for the given point in relation to the center of this region.
    #[inline]
    #[must_use]
    pub fn quadrant_for_ipoint<P>(&self, point: P) -> Quadrant
    where
        P: Into<IVec2>,
    {
        let point = point.into();
        let center = num_traits::cast(self.half_size()).unwrap();
        Quadrant::for_ipoint(point - self.point().as_ivec2(), center)
    }

    #[inline]
    #[must_use]
    pub fn intersect(&self, other: &URect) -> URect {
        let min = self.point();
        let max = self.end_point();
        let mut r = URect {
            min: min.max(other.min),
            max: max.min(other.max),
        };
        r.min = r.min.min(r.max);
        r
    }

    #[inline]
    #[must_use]
    pub fn as_urect(&self) -> URect {
        self.into()
    }
}

#[allow(clippy::from_over_into)]
impl<U: Unsigned + NumCast + Copy> Into<URect> for Region<U> {
    #[inline]
    #[must_use]
    fn into(self) -> URect {
        URect::from_corners(self.point(), self.end_point())
    }
}

#[allow(clippy::from_over_into)]
impl<U: Unsigned + NumCast + Copy> Into<URect> for &Region<U> {
    #[inline]
    #[must_use]
    fn into(self) -> URect {
        URect::from_corners(self.point(), self.end_point())
    }
}

#[allow(clippy::from_over_into)]
impl<U: Unsigned + NumCast + Copy> Into<IRect> for Region<U> {
    #[inline]
    #[must_use]
    fn into(self) -> IRect {
        IRect::from_corners(self.point().as_ivec2(), self.end_point().as_ivec2())
    }
}

#[allow(clippy::from_over_into)]
impl<U: Unsigned + NumCast + Copy> Into<IRect> for &Region<U> {
    #[inline]
    #[must_use]
    fn into(self) -> IRect {
        IRect::from_corners(self.point().as_ivec2(), self.end_point().as_ivec2())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_contains_upoint() {
        let r = Region::new(0u32, 0, 4);
        assert!(r.contains_upoint((0, 0)));
        assert!(r.contains_upoint((1, 0)));
        assert!(r.contains_upoint((2, 0)));
        assert!(r.contains_upoint((3, 0)));
        assert!(r.contains_upoint((0, 1)));
        assert!(r.contains_upoint((1, 1)));
        assert!(r.contains_upoint((2, 1)));
        assert!(r.contains_upoint((3, 1)));
        assert!(r.contains_upoint((0, 2)));
        assert!(r.contains_upoint((1, 2)));
        assert!(r.contains_upoint((2, 2)));
        assert!(r.contains_upoint((3, 2)));
        assert!(r.contains_upoint((0, 3)));
        assert!(r.contains_upoint((1, 3)));
        assert!(r.contains_upoint((2, 3)));
        assert!(r.contains_upoint((3, 3)));
        assert!(!r.contains_upoint((4, 0)));
        assert!(!r.contains_upoint((0, 4)));
    }

    #[test]
    fn test_quadrant_for_upoint() {
        let r = Region::new(0u32, 0, 4);
        assert_eq!(r.quadrant_for_upoint((0, 0)), Quadrant::BottomLeft);
        assert_eq!(r.quadrant_for_upoint((1, 0)), Quadrant::BottomLeft);
        assert_eq!(r.quadrant_for_upoint((2, 0)), Quadrant::BottomRight);
        assert_eq!(r.quadrant_for_upoint((3, 0)), Quadrant::BottomRight);
        assert_eq!(r.quadrant_for_upoint((0, 1)), Quadrant::BottomLeft);
        assert_eq!(r.quadrant_for_upoint((1, 1)), Quadrant::BottomLeft);
        assert_eq!(r.quadrant_for_upoint((2, 1)), Quadrant::BottomRight);
        assert_eq!(r.quadrant_for_upoint((3, 1)), Quadrant::BottomRight);
        assert_eq!(r.quadrant_for_upoint((0, 2)), Quadrant::TopLeft);
        assert_eq!(r.quadrant_for_upoint((1, 2)), Quadrant::TopLeft);
        assert_eq!(r.quadrant_for_upoint((2, 2)), Quadrant::TopRight);
        assert_eq!(r.quadrant_for_upoint((3, 2)), Quadrant::TopRight);
        assert_eq!(r.quadrant_for_upoint((0, 3)), Quadrant::TopLeft);
        assert_eq!(r.quadrant_for_upoint((1, 3)), Quadrant::TopLeft);
        assert_eq!(r.quadrant_for_upoint((2, 3)), Quadrant::TopRight);
        assert_eq!(r.quadrant_for_upoint((3, 3)), Quadrant::TopRight);
    }
}
