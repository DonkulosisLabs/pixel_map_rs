use super::quadrant::Quadrant;
use super::Point;
use num_traits::{NumCast, Unsigned};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Region<U: Unsigned + Copy = u16> {
    x: U,
    y: U,
    size: U,
}

impl<U: Unsigned + NumCast + Copy> Region<U> {
    #[inline]
    pub fn new(x: U, y: U, size: U) -> Self {
        Self { x, y, size }
    }

    #[inline]
    pub fn x(&self) -> U {
        self.x
    }

    #[inline]
    pub fn y(&self) -> U {
        self.y
    }

    #[inline]
    pub fn point(&self) -> Point {
        let x = num_traits::cast::cast(self.x).unwrap();
        let y = num_traits::cast::cast(self.y).unwrap();
        (x, y).into()
    }

    #[inline]
    pub fn end_point(&self) -> Point {
        let size: i32 = num_traits::cast::cast(self.size).unwrap();
        self.point() + size - 1
    }

    #[inline]
    pub fn size(&self) -> U {
        self.size
    }

    #[inline]
    pub fn center(&self) -> U {
        self.size / num_traits::cast::cast(2).unwrap()
    }

    #[inline]
    pub fn is_unit(&self, pixel_size: u8) -> bool {
        self.size == num_traits::cast(pixel_size).unwrap()
    }

    #[inline]
    pub fn contains<P>(&self, point: P) -> bool
    where
        P: Into<Point>,
    {
        let point = point.into();
        let x: i32 = num_traits::cast(self.x).unwrap();
        let y: i32 = num_traits::cast(self.y).unwrap();
        let size: i32 = num_traits::cast(self.size).unwrap();
        point.x() >= x && point.x() < x + size && point.y() >= y && point.y() < y + size
    }

    #[inline]
    pub fn quadrant_for<P>(&self, point: P) -> Quadrant
    where
        P: Into<Point>,
    {
        let point = point.into();
        let center = num_traits::cast(self.center()).unwrap();
        Quadrant::for_point(point - self.point(), center)
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
