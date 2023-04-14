use super::Direction;
use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub(super) x: i32,
    pub(super) y: i32,
}

impl Point {
    pub const ZERO: Self = Self { x: 0, y: 0 };
    pub const ONE: Self = Self { x: 1, y: 1 };
    pub const NEG_ONE: Self = Self { x: -1, y: -1 };
    pub const NORTH: Self = Self { x: 0, y: 1 };
    pub const NORTH_EAST: Self = Self { x: 1, y: 1 };
    pub const NORTH_WEST: Self = Self { x: -1, y: 1 };
    pub const EAST: Self = Self { x: 1, y: 0 };
    pub const SOUTH: Self = Self { x: 0, y: -1 };
    pub const SOUTH_EAST: Self = Self { x: 1, y: -1 };
    pub const SOUTH_WEST: Self = Self { x: -1, y: -1 };
    pub const WEST: Self = Self { x: -1, y: 0 };

    #[inline]
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn x(&self) -> i32 {
        self.x
    }

    #[inline]
    pub fn y(&self) -> i32 {
        self.y
    }

    #[inline]
    pub fn move_towards(&self, direction: Direction, by: i32) -> Self {
        let unit = direction.unit();
        *self + (unit * by)
    }

    #[inline]
    pub fn distance_squared_to(&self, other: Point) -> f32 {
        let x = other.x() as f32 - self.x() as f32;
        let y = other.y() as f32 - self.y() as f32;
        (x * x + y * y).abs()
    }

    #[inline]
    pub fn distance_to(&self, other: Point) -> f32 {
        self.distance_squared_to(other).sqrt()
    }

    #[inline]
    pub fn min(&self, other: Point) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

    #[inline]
    pub fn max(&self, other: Point) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }
}

impl From<(i32, i32)> for Point {
    #[inline]
    fn from((x, y): (i32, i32)) -> Self {
        Self::new(x, y)
    }
}

impl From<[i32; 2]> for Point {
    #[inline]
    fn from([x, y]: [i32; 2]) -> Self {
        Self::new(x, y)
    }
}

impl From<Point> for (i32, i32) {
    #[inline]
    fn from(point: Point) -> Self {
        (point.x, point.y)
    }
}

impl From<Point> for [i32; 2] {
    #[inline]
    fn from(point: Point) -> Self {
        [point.x, point.y]
    }
}

impl Neg for Point {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

impl Add for Point {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Add<i32> for Point {
    type Output = Self;

    #[inline]
    fn add(self, rhs: i32) -> Self::Output {
        Self::new(self.x + rhs, self.y + rhs)
    }
}

impl Sub for Point {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Sub<i32> for Point {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: i32) -> Self::Output {
        Self::new(self.x - rhs, self.y - rhs)
    }
}

impl Mul for Point {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl Mul<i32> for Point {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: i32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Div for Point {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        Self::new(self.x / rhs.x, self.y / rhs.y)
    }
}

impl Div<i32> for Point {
    type Output = Self;

    #[inline]
    fn div(self, rhs: i32) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs)
    }
}
