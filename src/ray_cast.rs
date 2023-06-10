use super::ILine;
use super::LinePixelIterator;
use glam::IVec2;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct RayCastQuery {
    pub line: ILine,
}

impl RayCastQuery {
    pub fn new(line: ILine) -> Self {
        Self { line }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayCastResult {
    pub collision_point: Option<IVec2>,
    pub distance: f32,
    pub traversed: u32,
}

impl RayCastResult {
    #[inline]
    pub fn is_hit(&self) -> bool {
        self.collision_point.is_some()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct RayCastContext {
    pub(super) line_iter: LinePixelIterator,
    pub(super) traversed: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RayCast {
    Continue,
    Hit,
}
