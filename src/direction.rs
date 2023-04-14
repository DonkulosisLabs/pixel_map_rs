use super::Point;

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
    pub fn unit(&self) -> Point {
        match self {
            Direction::North => Point::NORTH,
            Direction::NorthEast => Point::NORTH_EAST,
            Direction::NorthWest => Point::NORTH_WEST,
            Direction::East => Point::EAST,
            Direction::South => Point::SOUTH,
            Direction::SouthEast => Point::SOUTH_EAST,
            Direction::SouthWest => Point::SOUTH_WEST,
            Direction::West => Point::WEST,
        }
    }
}
