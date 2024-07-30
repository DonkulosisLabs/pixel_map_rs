#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

use crate::Quadrant;
use std::ops::Deref;

/// A path to a node in the pixel map.
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodePath(u64);

impl NodePath {
    pub const MASK: u64 = 0xffffff;
    pub const DEPTH: u64 = 48;

    pub const ROOT: NodePath = NodePath(0);

    #[inline]
    #[must_use]
    pub fn from_quadrants(quadrants: &[Quadrant]) -> NodePath {
        let mut path = Self::ROOT;
        for &quadrant in quadrants {
            path = path.append(quadrant);
        }
        path
    }

    #[inline]
    #[must_use]
    pub fn is_root(&self) -> bool {
        *self == Self::ROOT
    }

    #[inline]
    #[must_use]
    pub fn encode(depth: u16, path: u64) -> NodePath {
        NodePath(((depth as u64) << Self::DEPTH) | (path & Self::MASK))
    }

    #[inline]
    #[must_use]
    pub fn depth(&self) -> u16 {
        (self.0 >> Self::DEPTH) as u16
    }

    #[inline]
    #[must_use]
    pub fn path_bits(&self) -> u64 {
        self.0 & Self::MASK
    }

    #[inline]
    #[must_use]
    pub fn components(&self) -> (u16, u64) {
        (self.depth(), self.path_bits())
    }

    #[inline]
    #[must_use]
    pub fn quadrant_at(&self, index: u16) -> Option<Quadrant> {
        let self_depth = self.depth();
        if index >= self_depth {
            return None;
        }
        let path_bits = self.path_bits();
        let shift = index as u64 * 2;
        let quadrant_bits = (path_bits >> shift) & 0b11;
        Quadrant::from_value(quadrant_bits as u8)
    }

    #[inline]
    #[must_use]
    pub fn tail(&self) -> Option<Quadrant> {
        if *self == Self::ROOT {
            return None;
        }
        self.quadrant_at(self.depth() - 1)
    }

    #[inline]
    #[must_use]
    pub fn append(&self, quadrant: Quadrant) -> NodePath {
        let (depth, path) = self.components();
        let new_depth = depth + 1;
        let new_path = path | ((quadrant as u64) << (2 * depth));
        Self::encode(new_depth, new_path)
    }

    #[inline]
    #[must_use]
    pub fn parent(&self) -> NodePath {
        self.truncate(1)
    }

    #[inline]
    #[must_use]
    pub fn truncate(&self, count: u16) -> NodePath {
        // Special case: count is zero
        if count == 0 {
            return *self;
        }

        let mut node = *self;
        for _ in 0..count {
            let (depth, path) = node.components();
            if depth == 0 {
                return node;
            }

            let parent_depth = depth - 1;
            // Erase the bits of the current path element
            let parent_path = path & (!(0b11 << (2 * parent_depth)));

            node = Self::encode(parent_depth, parent_path)
        }

        node
    }

    #[must_use]
    pub fn common_ancestor(&self, b: NodePath) -> NodePath {
        let a = *self;

        // Special case: paths are equal
        if a == b {
            return a;
        }
        // Special case: either path is a root node
        if a.is_root() || b.is_root() {
            return Self::ROOT;
        }

        let a_depth = a.depth();
        let b_depth = b.depth();

        // Special case: if either path depth is 1, the common ancestor is the root
        if a_depth == 1 || b_depth == 1 {
            return Self::ROOT;
        }

        let min_depth = a_depth.min(b_depth) as u64;

        let a_path = a.path_bits();
        let b_path = b.path_bits();

        let mut index = 0u64;

        loop {
            if index >= min_depth {
                return Self::ROOT;
            }

            let mask = 0b11 << (index * 2);
            index += 1;
            if a_path & mask == b_path & mask {
                continue;
            }

            let shift = 64 - (index * 2) + 2;
            if shift >= 64 {
                return Self::ROOT;
            }

            let mask = (!0) >> shift;
            return Self::encode(index as u16 - 1, a_path & mask);
        }
    }
}

impl Deref for NodePath {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Quadrant> for NodePath {
    fn from(quadrant: Quadrant) -> Self {
        Self::encode(1, quadrant as u64)
    }
}

#[cfg(test)]
mod test {
    use crate::{NodePath, Quadrant};

    #[test]
    fn test_parent() {
        assert_eq!(NodePath::ROOT.parent(), NodePath::ROOT);

        let path = NodePath::encode(1, 0b01);
        assert_eq!(path.parent(), NodePath::ROOT);

        let path = NodePath::encode(2, 0b0101);
        assert_eq!(path.parent(), NodePath::encode(1, 0b01));

        let path = NodePath::encode(3, 0b010101);
        assert_eq!(path.parent(), NodePath::encode(2, 0b0101));

        let path = NodePath::encode(4, 0b01010101);
        assert_eq!(path.parent(), NodePath::encode(3, 0b010101));

        let path = NodePath::encode(5, 0b0101010101);
        assert_eq!(path.parent(), NodePath::encode(4, 0b01010101));

        let path = NodePath::encode(6, 0b010101010101);
        assert_eq!(path.parent(), NodePath::encode(5, 0b0101010101));

        let path = NodePath::encode(7, 0b01010101010101);
        assert_eq!(path.parent(), NodePath::encode(6, 0b010101010101));

        let path = NodePath::encode(8, 0b0101010101010101);
        assert_eq!(path.parent(), NodePath::encode(7, 0b01010101010101));

        let path = NodePath::encode(9, 0b010101010101010101);
        assert_eq!(path.parent(), NodePath::encode(8, 0b0101010101010101));

        let path = NodePath::encode(10, 0b01010101010101010101);
        assert_eq!(path.parent(), NodePath::encode(9, 0b010101010101010101));
    }

    #[test]
    fn test_truncate() {
        assert_eq!(NodePath::ROOT.parent(), NodePath::ROOT);

        let path = NodePath::encode(1, 0b01);
        assert_eq!(path.truncate(1), NodePath::ROOT);

        let path = NodePath::encode(1, 0b01);
        assert_eq!(path.truncate(2), NodePath::ROOT);

        let path = NodePath::encode(2, 0b0101);
        assert_eq!(path.truncate(1), NodePath::encode(1, 0b01));

        let path = NodePath::encode(2, 0b0101);
        assert_eq!(path.truncate(2), NodePath::ROOT);

        let path = NodePath::encode(2, 0b0101);
        assert_eq!(path.truncate(3), NodePath::ROOT);

        let path = NodePath::encode(3, 0b110101);
        assert_eq!(path.truncate(2), NodePath::encode(1, 0b01));
    }

    #[test]
    fn test_append() {
        let path = NodePath::ROOT.append(Quadrant::TopLeft);
        assert_eq!(path, NodePath::encode(1, 0b11));

        let path = NodePath::encode(4, 0b00_11_10_01);
        let path = path.append(Quadrant::TopLeft);
        assert_eq!(path, NodePath::encode(5, 0b11_00_11_10_01));
        let path = path.append(Quadrant::BottomRight);
        assert_eq!(path, NodePath::encode(6, 0b01_11_00_11_10_01));
    }

    #[test]
    fn test_quadrant_at() {
        let path = NodePath::encode(4, 0b00_11_10_01);
        assert_eq!(path.quadrant_at(0).unwrap(), Quadrant::BottomRight);
        assert_eq!(path.quadrant_at(1).unwrap(), Quadrant::TopRight);
        assert_eq!(path.quadrant_at(2).unwrap(), Quadrant::TopLeft);
        assert_eq!(path.quadrant_at(3).unwrap(), Quadrant::BottomLeft);
        assert_eq!(path.quadrant_at(4), None);
    }

    #[test]
    fn test_tail() {
        let path = NodePath::encode(4, 0b01_11_11_11);
        assert_eq!(path.tail().unwrap(), Quadrant::BottomRight);

        let path = NodePath::ROOT;
        assert_eq!(path.tail(), None);
    }

    #[test]
    fn test_common_ancestor() {
        assert_eq!(
            NodePath::ROOT.common_ancestor(NodePath::ROOT),
            NodePath::ROOT
        );

        let path_a = NodePath::encode(3, 0b01_11_11);
        let path_b = NodePath::encode(2, 0b01_11);
        assert_eq!(path_a.common_ancestor(path_b), NodePath::encode(1, 0b11));

        let path_a = NodePath::encode(2, 0b01_11);
        let path_b = NodePath::encode(3, 0b01_11_11);
        assert_eq!(path_a.common_ancestor(path_b), NodePath::encode(1, 0b11));

        let path_a = NodePath::encode(3, 0b01_11_01);
        let path_b = NodePath::encode(2, 0b01_11);
        assert_eq!(path_a.common_ancestor(path_b), NodePath::ROOT);

        let path_a = NodePath::encode(2, 0b01_11);
        let path_b = NodePath::encode(3, 0b01_11_01);
        assert_eq!(path_a.common_ancestor(path_b), NodePath::ROOT);

        let path_a = NodePath::encode(4, 0b01_11_11_11);
        let path_b = NodePath::encode(3, 0b01_11_11);
        assert_eq!(path_a.common_ancestor(path_b), NodePath::encode(2, 0b11_11));

        let path_a = NodePath::encode(3, 0b01_11_11);
        let path_b = NodePath::encode(4, 0b01_11_11_11);
        assert_eq!(path_a.common_ancestor(path_b), NodePath::encode(2, 0b11_11));

        let path_a = NodePath::encode(4, 0b01_11_01_11);
        let path_b = NodePath::encode(3, 0b01_11_11);
        assert_eq!(path_a.common_ancestor(path_b), NodePath::encode(1, 0b11));

        let path_a = NodePath::encode(3, 0b01_11_11);
        let path_b = NodePath::encode(4, 0b01_11_01_11);
        assert_eq!(path_a.common_ancestor(path_b), NodePath::encode(1, 0b11));
    }
}
