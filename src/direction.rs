#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use bevy_math::IVec2;

pub const NORTH: IVec2 = IVec2 { x: 0, y: 1 };
pub const NORTH_EAST: IVec2 = IVec2 { x: 1, y: 1 };
pub const NORTH_WEST: IVec2 = IVec2 { x: -1, y: 1 };
pub const EAST: IVec2 = IVec2 { x: 1, y: 0 };
pub const SOUTH: IVec2 = IVec2 { x: 0, y: -1 };
pub const SOUTH_EAST: IVec2 = IVec2 { x: 1, y: -1 };
pub const SOUTH_WEST: IVec2 = IVec2 { x: -1, y: -1 };
pub const WEST: IVec2 = IVec2 { x: -1, y: 0 };

/// A direction in the 2D plane.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    NorthEast,
    NorthWest,
    East,
    South,
    SouthEast,
    SouthWest,
    West,
}

impl Direction {
    /// Returns the unit vector for this direction.
    #[inline]
    #[must_use]
    pub fn unit(&self) -> IVec2 {
        match self {
            Direction::North => NORTH,
            Direction::NorthEast => NORTH_EAST,
            Direction::NorthWest => NORTH_WEST,
            Direction::East => EAST,
            Direction::South => SOUTH,
            Direction::SouthEast => SOUTH_EAST,
            Direction::SouthWest => SOUTH_WEST,
            Direction::West => WEST,
        }
    }

    /// Move a point in this direction by the given amount.
    #[inline]
    #[must_use]
    pub fn move_point(&self, point: IVec2, by: i32) -> IVec2 {
        point + self.unit() * by
    }

    /// Returns true if this direction is cardinal (N, E, S, W).
    #[inline]
    #[must_use]
    pub fn is_cardinal(&self) -> bool {
        matches!(
            self,
            Direction::North | Direction::East | Direction::South | Direction::West
        )
    }

    /// Returns true if this direction is diagonal (NE, NW, SE, SW).
    #[inline]
    #[must_use]
    pub fn is_diagonal(&self) -> bool {
        matches!(
            self,
            Direction::NorthEast
                | Direction::NorthWest
                | Direction::SouthEast
                | Direction::SouthWest
        )
    }
}
