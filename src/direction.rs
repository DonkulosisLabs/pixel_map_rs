#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

use bevy_math::{uvec2, IVec2, UVec2};

pub const NORTH: IVec2 = IVec2 { x: 0, y: 1 };
pub const NORTH_EAST: IVec2 = IVec2 { x: 1, y: 1 };
pub const NORTH_WEST: IVec2 = IVec2 { x: -1, y: 1 };
pub const EAST: IVec2 = IVec2 { x: 1, y: 0 };
pub const SOUTH: IVec2 = IVec2 { x: 0, y: -1 };
pub const SOUTH_EAST: IVec2 = IVec2 { x: 1, y: -1 };
pub const SOUTH_WEST: IVec2 = IVec2 { x: -1, y: -1 };
pub const WEST: IVec2 = IVec2 { x: -1, y: 0 };

/// A direction in the 2D plane.
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
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
    /// Iterate N, E, S, W directions.
    #[inline]
    pub fn iter_cardinal() -> impl Iterator<Item = Direction> {
        [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ]
        .iter()
        .copied()
    }

    /// Iterate NE, NW, SE, SW directions.
    #[inline]
    pub fn iter_diagonal() -> impl Iterator<Item = Direction> {
        [
            Direction::NorthEast,
            Direction::NorthWest,
            Direction::SouthEast,
            Direction::SouthWest,
        ]
        .iter()
        .copied()
    }

    /// Iterate all directions.
    #[inline]
    pub fn iter() -> impl Iterator<Item = Direction> {
        [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
            Direction::NorthEast,
            Direction::NorthWest,
            Direction::SouthEast,
            Direction::SouthWest,
        ]
        .iter()
        .copied()
    }

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
    pub fn move_ipoint(&self, point: IVec2, by: i32) -> IVec2 {
        point + self.unit() * by
    }

    /// Move a point in this direction by the given amount.
    /// Limits the lower bounds to zero.
    #[inline]
    #[must_use]
    pub fn move_upoint(&self, point: UVec2, by: u32) -> UVec2 {
        let r = point.as_ivec2() + self.unit() * by as i32;
        if r.x < 0 || r.y < 0 {
            UVec2::default()
        } else if r.x < 0 {
            uvec2(0, r.y as u32)
        } else if r.y < 0 {
            uvec2(r.x as u32, 0)
        } else {
            r.as_uvec2()
        }
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
