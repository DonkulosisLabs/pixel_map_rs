use super::IVec2;

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
    pub fn unit(&self) -> IVec2 {
        match self {
            Direction::North => IVec2::NORTH,
            Direction::NorthEast => IVec2::NORTH_EAST,
            Direction::NorthWest => IVec2::NORTH_WEST,
            Direction::East => IVec2::EAST,
            Direction::South => IVec2::SOUTH,
            Direction::SouthEast => IVec2::SOUTH_EAST,
            Direction::SouthWest => IVec2::SOUTH_WEST,
            Direction::West => IVec2::WEST,
        }
    }
}
