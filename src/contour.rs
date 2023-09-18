use bevy_math::UVec2;

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Edge(pub UVec2, pub UVec2);

impl Edge {
    #[inline]
    #[must_use]
    pub fn new(a: UVec2, b: UVec2) -> Self {
        Self(a, b)
    }
}
