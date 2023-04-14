use super::line_iterator::LineIterator;
use super::{Line, Point};

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct RayCastQuery {
    pub line: Line,
}

impl RayCastQuery {
    pub fn new(line: Line) -> Self {
        Self { line }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayCastResult {
    pub collision_point: Option<Point>,
    pub distance: f32,
    pub traversed: u32,
}

impl RayCastResult {
    pub fn is_hit(&self) -> bool {
        self.collision_point.is_some()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct RayCastContext {
    pub(super) line_iter: LineIterator,
    pub(super) traversed: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RayCast {
    Continue,
    Hit,
}
