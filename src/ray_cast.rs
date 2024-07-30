#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

use super::ILine;
use super::LinePixelIterator;
use bevy_math::UVec2;

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct RayCastQuery {
    pub line: ILine,
}

impl RayCastQuery {
    #[inline]
    #[must_use]
    pub fn new(line: ILine) -> Self {
        Self { line }
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayCastResult {
    pub collision_point: Option<UVec2>,
    pub distance: f32,
    pub traversed: u32,
}

impl RayCastResult {
    #[inline]
    #[must_use]
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
