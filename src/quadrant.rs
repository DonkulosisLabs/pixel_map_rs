#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::Direction;
use bevy_math::{IVec2, UVec2};

/// A quadrant in a box.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum Quadrant {
    BottomLeft = 0,
    BottomRight = 1,
    TopRight = 2,
    TopLeft = 3,
}

impl Quadrant {
    /// Obtain an iterator over all [Quadrant] variants.
    #[inline]
    pub fn iter() -> impl Iterator<Item = Quadrant> {
        [
            Quadrant::BottomLeft,
            Quadrant::BottomRight,
            Quadrant::TopRight,
            Quadrant::TopLeft,
        ]
        .into_iter()
    }

    /// The bit representation of the quadrant, which can be used in a bitmask.
    #[inline]
    pub const fn as_bit(&self) -> u8 {
        1 << *self as u8
    }

    /// Returns the quadrant for the given point in relation to the given center point.
    #[inline]
    #[must_use]
    pub fn for_upoint<P>(point: P, center: u32) -> Quadrant
    where
        P: Into<UVec2>,
    {
        let point = point.into();
        if point.x < center {
            if point.y >= center {
                Quadrant::TopLeft
            } else {
                Quadrant::BottomLeft
            }
        } else if point.y >= center {
            Quadrant::TopRight
        } else {
            Quadrant::BottomRight
        }
    }

    /// Returns the quadrant for the given point in relation to the given center point.
    #[inline]
    #[must_use]
    pub fn for_ipoint<P>(point: P, center: i32) -> Quadrant
    where
        P: Into<IVec2>,
    {
        let point = point.into();
        if point.x < center {
            if point.y >= center {
                Quadrant::TopLeft
            } else {
                Quadrant::BottomLeft
            }
        } else if point.y >= center {
            Quadrant::TopRight
        } else {
            Quadrant::BottomRight
        }
    }

    #[inline]
    #[must_use]
    pub fn from_value(value: u8) -> Option<Quadrant> {
        match value {
            0 => Some(Quadrant::BottomLeft),
            1 => Some(Quadrant::BottomRight),
            2 => Some(Quadrant::TopRight),
            3 => Some(Quadrant::TopLeft),
            _ => None,
        }
    }

    /// Obtains the neighboring quadrant in the given direction, if there is one.
    #[inline]
    #[must_use]
    pub fn neighbor(&self, direction: Direction) -> Option<Quadrant> {
        match self {
            Quadrant::BottomLeft => match direction {
                Direction::North => Some(Quadrant::TopLeft),
                Direction::East => Some(Quadrant::BottomRight),
                Direction::NorthEast => Some(Quadrant::TopRight),
                _ => None,
            },
            Quadrant::BottomRight => match direction {
                Direction::North => Some(Quadrant::TopRight),
                Direction::West => Some(Quadrant::BottomLeft),
                Direction::NorthWest => Some(Quadrant::TopRight),
                _ => None,
            },
            Quadrant::TopRight => match direction {
                Direction::South => Some(Quadrant::BottomRight),
                Direction::West => Some(Quadrant::TopLeft),
                Direction::SouthWest => Some(Quadrant::BottomLeft),
                _ => None,
            },
            Quadrant::TopLeft => match direction {
                Direction::East => Some(Quadrant::TopRight),
                Direction::South => Some(Quadrant::BottomLeft),
                Direction::SouthEast => Some(Quadrant::BottomRight),
                _ => None,
            },
        }
    }
}

/// A [PixelMap] quad tree node fill pattern, regarding child node storage.
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum PNodeFill {
    /// ```text
    /// oo
    /// oo
    /// ```
    Empty = 0,

    /// ```text
    /// xx
    /// xx
    /// ```
    Full = 0b1111,

    /// ```text
    /// xo
    /// oo
    /// ```
    TopLeft = Quadrant::TopLeft.as_bit(),

    /// ```text
    /// ox
    /// oo
    /// ```
    TopRight = Quadrant::TopRight.as_bit(),

    /// ```text
    /// oo
    /// xo
    /// ```
    BottomLeft = Quadrant::BottomLeft.as_bit(),

    /// ```text
    /// oo
    /// ox
    /// ```
    BottomRight = Quadrant::BottomRight.as_bit(),

    /// ```text
    /// xx
    /// oo
    /// ```
    Top = Quadrant::TopLeft.as_bit() | Quadrant::TopRight.as_bit(),

    /// ```text
    /// oo
    /// xx
    /// ```
    Bottom = Quadrant::BottomLeft.as_bit() | Quadrant::BottomRight.as_bit(),

    /// ```text
    /// xo
    /// xo
    /// ```
    Left = Quadrant::BottomLeft.as_bit() | Quadrant::TopLeft.as_bit(),

    /// ```text
    /// ox
    /// ox
    /// ```
    Right = Quadrant::BottomRight.as_bit() | Quadrant::TopRight.as_bit(),

    /// ```text
    /// ox
    /// xo
    /// ```
    BottomRightTopLeft = Quadrant::BottomRight.as_bit() | Quadrant::TopLeft.as_bit(),

    /// ```text
    /// xo
    /// ox
    /// ```
    BottomLeftTopRight = Quadrant::BottomLeft.as_bit() | Quadrant::TopRight.as_bit(),

    /// ```text
    /// ox
    /// xx
    /// ```
    NotTopLeft = Quadrant::BottomLeft.as_bit()
        | Quadrant::BottomRight.as_bit()
        | Quadrant::TopRight.as_bit(),

    /// ```text
    /// xo
    /// xx
    /// ```
    NotTopRight =
        Quadrant::BottomLeft.as_bit() | Quadrant::BottomRight.as_bit() | Quadrant::TopLeft.as_bit(),

    /// ```text
    /// xx
    /// ox
    /// ```
    NotBottomLeft =
        Quadrant::BottomRight.as_bit() | Quadrant::TopLeft.as_bit() | Quadrant::TopRight.as_bit(),

    /// ```text
    /// xx
    /// xo
    /// ```
    NotBottomRight =
        Quadrant::BottomLeft.as_bit() | Quadrant::TopLeft.as_bit() | Quadrant::TopRight.as_bit(),
}

impl PNodeFill {
    /// Negate the node fill pattern.
    #[inline]
    pub fn invert(&self) -> PNodeFill {
        unsafe { std::mem::transmute(!(*self as u8) & 0b1111) }
    }

    /// If the fill represents a single quadrant, return that quadrant. `None`, otherwise.
    pub fn quadrant(&self) -> Option<Quadrant> {
        match self {
            PNodeFill::TopLeft => Some(Quadrant::TopLeft),
            PNodeFill::TopRight => Some(Quadrant::TopRight),
            PNodeFill::BottomLeft => Some(Quadrant::BottomLeft),
            PNodeFill::BottomRight => Some(Quadrant::BottomRight),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::Quadrant;
    use crate::PNodeFill;

    #[test]
    fn test_for_upoint() {
        assert_eq!(Quadrant::for_upoint((0, 0), 1), Quadrant::BottomLeft);
        assert_eq!(Quadrant::for_upoint((1, 0), 1), Quadrant::BottomRight);
        assert_eq!(Quadrant::for_upoint((0, 1), 1), Quadrant::TopLeft);
        assert_eq!(Quadrant::for_upoint((1, 1), 1), Quadrant::TopRight);
    }

    #[test]
    fn test_for_ipoint() {
        assert_eq!(Quadrant::for_ipoint((0, 0), 1), Quadrant::BottomLeft);
        assert_eq!(Quadrant::for_ipoint((1, 0), 1), Quadrant::BottomRight);
        assert_eq!(Quadrant::for_ipoint((0, 1), 1), Quadrant::TopLeft);
        assert_eq!(Quadrant::for_ipoint((1, 1), 1), Quadrant::TopRight);
    }

    #[test]
    fn test_node_fill_invert() {
        assert_eq!(PNodeFill::Full.invert(), PNodeFill::Empty);
        assert_eq!(PNodeFill::Empty.invert(), PNodeFill::Full);
        assert_eq!(PNodeFill::Left.invert(), PNodeFill::Right);
        assert_eq!(PNodeFill::Right.invert(), PNodeFill::Left);
        assert_eq!(PNodeFill::TopLeft.invert(), PNodeFill::NotTopLeft);
        assert_eq!(PNodeFill::NotTopLeft.invert(), PNodeFill::TopLeft);
    }
}
