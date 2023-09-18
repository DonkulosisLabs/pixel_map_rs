#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{ICircle, RayCast, RayCastContext, RayCastQuery, RayCastResult, Region};
use crate::{distance_to, exclusive_urect, NodePath};
use bevy_math::{URect, UVec2};
use num_traits::{NumCast, Unsigned};
use std::fmt::Debug;

pub type Children<T, U> = Box<[PNode<T, U>; 4]>;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
enum PNodeKind<T: Copy + PartialEq = bool, U: Unsigned + NumCast + Copy + Debug = u16> {
    Leaf(T),
    Branch(Children<T, U>),
}

impl<T: Copy + PartialEq, U: Unsigned + NumCast + Copy + Debug> PNodeKind<T, U> {
    #[inline]
    pub fn value(&self) -> &T {
        match self {
            PNodeKind::Leaf(value) => value,
            PNodeKind::Branch(_) => {
                panic!("pixel map leaf node value accessed in branch node context");
            }
        }
    }

    #[inline]
    pub fn children(&self) -> &Children<T, U> {
        match self {
            PNodeKind::Leaf(_) => {
                panic!("pixel map branch node children accessed in leaf node context");
            }
            PNodeKind::Branch(children) => children,
        }
    }

    #[inline]
    pub fn children_mut(&mut self) -> &mut Children<T, U> {
        match self {
            PNodeKind::Leaf(_) => {
                panic!("pixel map branch node children accessed in leaf node context");
            }
            PNodeKind::Branch(children) => children,
        }
    }
}

/// A node of a [crate::PixelMap] quad tree.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct PNode<T: Copy + PartialEq = bool, U: Unsigned + NumCast + Copy + Debug = u16> {
    region: Region<U>,
    kind: PNodeKind<T, U>,
    dirty: bool,
}

impl<T: Copy + PartialEq, U: Unsigned + NumCast + Copy + Debug> PNode<T, U> {
    #[inline]
    #[must_use]
    pub(super) fn new(region: Region<U>, value: T, dirty: bool) -> Self {
        Self {
            region,
            kind: PNodeKind::Leaf(value),
            dirty,
        }
    }

    /// Obtain the region represented by this node.
    #[inline]
    #[must_use]
    pub fn region(&self) -> &Region<U> {
        &self.region
    }

    /// Determine if this node is in a dirty state. This can be used to represent a
    /// modified node that needs to be manipulated in some way (i.e. written to an Image texture).
    #[inline]
    #[must_use]
    pub fn dirty(&self) -> bool {
        self.dirty
    }

    /// Set the dirty state of this node to false.
    #[inline]
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Obtain this node's value.
    /// Panics if this node is not [Self::is_leaf()].
    #[inline]
    #[must_use]
    pub fn value(&self) -> &T {
        self.kind.value()
    }

    /// Set the value of this node. If this node has children, they will be discarded.
    /// This marks the node as dirty.
    #[inline]
    pub(super) fn set_value(&mut self, value: T) {
        self.dirty = true;
        self.kind = PNodeKind::Leaf(value);
    }

    /// Obtain an array of the children of this node.
    /// Panics if this node is [Self::is_leaf()].
    #[inline]
    #[must_use]
    pub fn children(&self) -> &Children<T, U> {
        self.kind.children()
    }

    #[inline]
    #[must_use]
    fn children_mut(&mut self) -> &mut Children<T, U> {
        self.kind.children_mut()
    }

    /// Determine if this node is a leaf node. Leaves don't have children.
    #[inline]
    #[must_use]
    pub fn is_leaf(&self) -> bool {
        matches!(self.kind, PNodeKind::Leaf(_))
    }

    /// Determine if all immediate children of this node are leaf nodes.
    #[inline]
    #[must_use]
    pub fn is_leaf_parent(&self) -> bool {
        return match &self.kind {
            PNodeKind::Leaf(_) => false,
            PNodeKind::Branch(children) => children.iter().all(|c| c.is_leaf()),
        };
    }

    // Visit all nodes.
    pub(super) fn visit_nodes<F>(&self, visitor: &mut F)
    where
        F: FnMut(&PNode<T, U>),
    {
        visitor(self);
        if let PNodeKind::Branch(children) = &self.kind {
            for child in children.as_ref() {
                child.visit_nodes(visitor);
            }
        }
    }

    // Visit all leaf nodes within a given rectangle boundary.
    pub(super) fn visit_leaves_in_rect<F>(&self, rect: &URect, visitor: &mut F, traversed: &mut u32)
    where
        F: FnMut(&PNode<T, U>, &URect),
    {
        *traversed += 1;

        let sub_rect = self.region().intersect(rect);
        if !sub_rect.is_empty() {
            match self.kind {
                PNodeKind::Leaf(_) => visitor(self, &sub_rect),
                PNodeKind::Branch(ref children) => {
                    for child in children.as_ref() {
                        child.visit_leaves_in_rect(rect, visitor, traversed);
                    }
                }
            }
        }
    }

    pub(super) fn any_leaves_in_rect<F>(&self, rect: &URect, f: &mut F) -> Option<bool>
    where
        F: FnMut(&PNode<T, U>, &URect) -> bool,
    {
        let sub_rect = self.region().intersect(rect);
        if !sub_rect.is_empty() {
            match self.kind {
                PNodeKind::Branch(ref children) => {
                    for child in children.as_ref() {
                        if let Some(true) = child.any_leaves_in_rect(rect, f) {
                            return Some(true);
                        }
                    }
                }
                PNodeKind::Leaf(_) => {
                    if f(self, &sub_rect) {
                        return Some(true);
                    }
                }
            }
            return Some(false);
        }
        None
    }

    pub(super) fn all_leaves_in_rect<F>(&self, rect: &URect, f: &mut F) -> Option<bool>
    where
        F: FnMut(&PNode<T, U>, &URect) -> bool,
    {
        let sub_rect = self.region().intersect(rect);
        if !sub_rect.is_empty() {
            match self.kind {
                PNodeKind::Branch(ref children) => {
                    for child in children.as_ref() {
                        if let Some(false) = child.all_leaves_in_rect(rect, f) {
                            return Some(false);
                        }
                    }
                }
                PNodeKind::Leaf(_) => {
                    if !f(self, &sub_rect) {
                        return Some(false);
                    }
                }
            }
            return Some(true);
        }
        None
    }

    // This node must be known to be dirty.
    pub(super) fn visit_dirty_leaves_in_rect<F>(
        &self,
        rect: &URect,
        visitor: &mut F,
        traversed: &mut u32,
    ) where
        F: FnMut(&PNode<T, U>, &URect),
    {
        *traversed += 1;

        let sub_rect = self.region().intersect(rect);
        if !sub_rect.is_empty() {
            match self.kind {
                PNodeKind::Branch(ref children) => {
                    for child in children.as_ref() {
                        if child.dirty() {
                            child.visit_dirty_leaves_in_rect(rect, visitor, traversed);
                        }
                    }
                }
                PNodeKind::Leaf(_) => visitor(self, &sub_rect),
            }
        }
    }

    // This node must be known to be dirty.
    pub(super) fn drain_dirty_leaves<F>(&mut self, visitor: &mut F, traversed: &mut usize)
    where
        F: FnMut(&PNode<T, U>),
    {
        *traversed += 1;

        self.clear_dirty();
        match self.kind {
            PNodeKind::Branch(ref mut children) => {
                for child in children.as_mut() {
                    if child.dirty() {
                        child.drain_dirty_leaves(visitor, traversed);
                    }
                }
            }
            PNodeKind::Leaf(_) => visitor(self),
        }
    }

    // Get the node that contains the given coordinates. The coordinates must be
    // known to be within the bounds of this node.
    #[inline]
    #[must_use]
    pub(super) fn find_node(&self, point: UVec2) -> &PNode<T, U> {
        let mut node = self;
        loop {
            if let PNodeKind::Branch(children) = &node.kind {
                let q = node.region.quadrant_for(point);
                node = &children[q as usize];
            } else {
                return node;
            }
        }
    }

    #[inline]
    #[must_use]
    pub(super) fn node_path(&self, point: UVec2) -> (&PNode<T, U>, NodePath) {
        let mut depth = 0;
        let mut node = self;
        let mut path = 0;
        loop {
            if let PNodeKind::Branch(children) = &node.kind {
                let q = node.region.quadrant_for(point);
                path |= (q as u64) << (depth * 2);
                depth += 1;
                node = &children[q as usize];
            } else {
                depth += 1;
                return (node, NodePath::encode(depth, path));
            }
        }
    }

    #[inline]
    #[must_use]
    #[allow(dead_code)]
    pub(super) fn find_node_by_path(&self, path: NodePath) -> Option<&PNode<T, U>> {
        let mut path_depth = path.depth() as u64;
        if path_depth == 0 {
            return None;
        }
        path_depth -= 1; // make zero-based for bit indexing

        let mut node = self;
        let mut depth = 0u64;
        loop {
            if depth == path_depth {
                return Some(node);
            }
            if let PNodeKind::Branch(children) = &node.kind {
                let q = (*path >> (depth * 2)) & 0b11;
                depth += 1;
                node = &children[q as usize];
            } else {
                return None;
            }
        }
    }

    #[must_use]
    pub(super) fn ray_cast<F>(
        &self,
        query: &RayCastQuery,
        ctx: &mut RayCastContext,
        visitor: &mut F,
    ) -> Option<RayCastResult>
    where
        F: FnMut(&PNode<T, U>) -> RayCast,
    {
        loop {
            ctx.traversed += 1;
            let current_point = ctx.line_iter.peek()?;
            if self.region.contains(current_point) {
                match self.kind {
                    PNodeKind::Branch(ref children) => {
                        let q = self.region.quadrant_for(current_point);
                        let result = children[q as usize].ray_cast(query, ctx, visitor);
                        if result.is_some() {
                            return result;
                        }
                        continue;
                    }
                    PNodeKind::Leaf(_) => {
                        return match visitor(self) {
                            RayCast::Continue => {
                                ctx.line_iter.seek_bounds(&self.region().into());
                                continue;
                            }
                            RayCast::Hit => {
                                let distance = distance_to(query.line.start(), current_point);
                                let result = RayCastResult {
                                    collision_point: Some(current_point),
                                    distance,
                                    traversed: ctx.traversed,
                                };
                                Some(result)
                            }
                        };
                    }
                }
            } else {
                return None;
            }
        }
    }

    pub(super) fn set_pixel(&mut self, point: UVec2, pixel_size: u8, value: T) -> bool {
        if self.region.contains(point) {
            if self.is_leaf() && &value == self.value() {
                return true;
            }
            if self.region.is_unit(pixel_size) {
                self.set_value(value);
            } else {
                self.subdivide();
                let q = self.region.quadrant_for(point);
                self.children_mut()[q as usize].set_pixel(point, pixel_size, value);
                self.decimate();
                self.recalc_dirty();
            }
            return true;
        }
        false
    }

    pub(super) fn draw_rect(&mut self, rect: &URect, pixel_size: u8, value: T) {
        if self.contained_by_rect(rect) {
            self.set_value(value);
        } else {
            let sub_rect = self.region().intersect(rect);
            if !sub_rect.is_empty() {
                if self.is_leaf() && &value == self.value() {
                    return;
                }
                if self.region.is_unit(pixel_size) {
                    self.set_value(value);
                } else {
                    self.subdivide();
                    let children = self.children_mut();
                    children[0].draw_rect(&sub_rect, pixel_size, value);
                    children[1].draw_rect(&sub_rect, pixel_size, value);
                    children[2].draw_rect(&sub_rect, pixel_size, value);
                    children[3].draw_rect(&sub_rect, pixel_size, value);
                    self.decimate();
                    self.recalc_dirty();
                }
            }
        }
    }

    pub(super) fn draw_circle(&mut self, circle: &ICircle, pixel_size: u8, value: T) {
        let outer_rect = circle.aabb();
        let inner_rect = circle.inner_rect();
        if self.contained_by_rect(&inner_rect) {
            self.set_value(value);
        } else if !self.region().intersect(&outer_rect).is_empty() {
            self.draw_rect(&inner_rect, pixel_size, value);
            let inner_rect = exclusive_urect(&inner_rect);
            for p in circle.cropped_pixels() {
                if inner_rect.contains(p) {
                    continue;
                }
                self.set_pixel(p, pixel_size, value);
            }
        }
    }

    #[inline]
    #[must_use]
    fn contained_by_rect(&self, rect: &URect) -> bool {
        rect.contains(self.region.point()) && rect.contains(self.region.end_point())
    }

    fn subdivide(&mut self) {
        if !self.is_leaf() {
            return;
        }

        let x = self.region.x();
        let y = self.region.y();
        let half_size = self.region.center();

        let value = *self.value();
        self.kind = PNodeKind::Branch(Box::new([
            PNode::new(Region::new(x, y, half_size), value, self.dirty),
            PNode::new(Region::new(x + half_size, y, half_size), value, self.dirty),
            PNode::new(
                Region::new(x + half_size, y + half_size, half_size),
                value,
                self.dirty,
            ),
            PNode::new(Region::new(x, y + half_size, half_size), value, self.dirty),
        ]));
    }

    fn decimate(&mut self) {
        if !self.is_leaf_parent() {
            return;
        }

        if let PNodeKind::Branch(children) = &self.kind {
            let mut all_same = true;
            let mut c: Option<&T> = None;

            for child in children.iter() {
                if let Some(color) = c {
                    if color != child.value() {
                        all_same = false;
                        break;
                    }
                } else {
                    c = Some(child.value());
                }
            }

            if all_same {
                self.set_value(*c.unwrap());
            }
        }
    }

    #[inline]
    fn recalc_dirty(&mut self) {
        if let PNodeKind::Branch(children) = &self.kind {
            self.dirty = children.iter().any(|child| child.dirty);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Quadrant;

    #[test]
    fn test_subdivide() {
        let mut n = PNode::new(Region::new(0u32, 0, 4), false, false);
        n.subdivide();
        let children = n.children();
        assert_eq!(
            &children[0],
            &PNode::new(Region::new(0u32, 0, 2), false, false)
        );
        assert_eq!(
            &children[1],
            &PNode::new(Region::new(2u32, 0, 2), false, false)
        );
        assert_eq!(
            &children[2],
            &PNode::new(Region::new(2u32, 2, 2), false, false)
        );
        assert_eq!(
            &children[3],
            &PNode::new(Region::new(0u32, 2, 2), false, false)
        );
    }

    #[test]
    fn test_decimate_frees_children() {
        let mut n = PNode::new(Region::new(0u32, 0, 4), false, false);
        n.subdivide();
        assert!(!n.is_leaf());
        n.decimate();
        assert!(n.is_leaf());
    }

    #[test]
    fn test_decimate_retains_children() {
        let mut n = PNode::new(Region::new(0u32, 0, 4), false, false);
        n.subdivide();
        n.children_mut()[0].set_value(true);
        n.decimate();
        assert!(!n.is_leaf());
    }

    #[test]
    fn test_find_node() {
        let mut n = PNode::new(Region::new(0u32, 0, 2), false, false);
        n.subdivide();
        n.children_mut()[0].set_value(true);
        assert!(n.find_node((0, 0).into()).value());
        assert!(!n.find_node((1, 0).into()).value());
        assert!(!n.find_node((0, 1).into()).value());
        assert!(!n.find_node((1, 1).into()).value());
    }

    #[test]
    fn test_node_path() {
        let mut n = PNode::new(Region::new(0u32, 0, 4), false, false);

        let (_, path) = n.node_path((0, 0).into());
        assert_eq!(path.depth(), 1);

        n.subdivide();

        let (_, path) = n.node_path((0, 0).into());
        assert_eq!(path.path_bits(), 0);
        assert_eq!(path.depth(), 2);

        n.children_mut()[0].subdivide();

        let (_, path) = n.node_path((0, 0).into());
        assert_eq!(path.path_bits(), 0);
        assert_eq!(path.depth(), 3);

        let (_, path) = n.node_path((1, 1).into());
        assert_eq!(path.path_bits(), 0b1000);
        assert_eq!(path.depth(), 3);

        let (_, path) = n.node_path((2, 2).into());
        assert_eq!(path.path_bits(), 0b10);
        assert_eq!(path.depth(), 2);

        let (_, path) = n.node_path((3, 3).into());
        assert_eq!(path.path_bits(), 0b10);
        assert_eq!(path.depth(), 2);
    }

    #[test]
    fn test_find_node_by_path() {
        let mut n = PNode::new(Region::new(0u32, 0, 4), false, false);

        let node = n.find_node_by_path(NodePath::ROOT);
        assert_eq!(node, None);

        let node = n.find_node_by_path(NodePath::encode(1, 0));
        assert_eq!(*node.unwrap(), n);

        let node = n.find_node_by_path(NodePath::encode(2, 0));
        assert_eq!(node, None);

        n.subdivide();

        let node = n.find_node_by_path(NodePath::ROOT);
        assert_eq!(node, None);

        let node = n.find_node_by_path(NodePath::encode(1, 0));
        assert_eq!(*node.unwrap(), n);

        let node = n.find_node_by_path(NodePath::encode(2, 0));
        assert_eq!(*node.unwrap(), n.children()[0]);

        let node = n.find_node_by_path(NodePath::encode(2, 0b01));
        assert_eq!(*node.unwrap(), n.children()[1]);

        let node = n.find_node_by_path(NodePath::encode(2, 0b10));
        assert_eq!(*node.unwrap(), n.children()[2]);

        let node = n.find_node_by_path(NodePath::encode(2, 0b11));
        assert_eq!(*node.unwrap(), n.children()[3]);

        let node = n.find_node_by_path(NodePath::encode(3, 0b11));
        assert_eq!(node, None);

        n.children_mut()[0].subdivide();

        let node = n.find_node_by_path(NodePath::encode(2, 0));
        assert_eq!(*node.unwrap(), n.children()[0]);

        let node = n.find_node_by_path(NodePath::encode(3, 0));
        assert_eq!(*node.unwrap(), n.children()[0].children()[0]);

        let node = n.find_node_by_path(NodePath::encode(4, 0));
        assert_eq!(node, None);
    }

    #[test]
    fn test_set_pixel_subdivides() {
        let mut n = PNode::new(Region::new(0u32, 0, 2), false, false);
        n.set_pixel((0, 0).into(), 1, true);
        assert!(!n.is_leaf());
        assert!(n.find_node((0, 0).into()).value());
        assert!(!n.find_node((1, 0).into()).value());
        assert!(!n.find_node((0, 1).into()).value());
        assert!(!n.find_node((1, 1).into()).value());
    }

    #[test]
    fn test_set_pixel_on_decimates() {
        let mut n = PNode::new(Region::new(0u32, 0, 2), false, false);
        n.set_pixel((0, 0).into(), 1, true);
        n.set_pixel((1, 0).into(), 1, true);
        n.set_pixel((0, 1).into(), 1, true);
        n.set_pixel((1, 1).into(), 1, true);
        assert!(n.value());
        assert!(n.is_leaf());
    }

    #[test]
    fn test_set_pixel_off_decimates() {
        let mut n = PNode::new(Region::new(0u32, 0, 2), false, false);
        n.set_pixel((0, 0).into(), 1, true);
        n.set_pixel((0, 0).into(), 1, false);
        assert!(!n.value());
        assert!(n.is_leaf());
    }

    #[test]
    fn test_visit_nodes() {
        let mut n = PNode::new(Region::new(0u32, 0, 2), false, false);
        n.set_pixel((0, 0).into(), 1, true);
        n.set_pixel((1, 0).into(), 1, false);
        n.set_pixel((0, 1).into(), 1, true);
        n.set_pixel((1, 1).into(), 1, false);
        let mut count = 0;
        n.visit_nodes(&mut |_n| {
            count += 1;
        });
        assert_eq!(count, 5);
    }

    #[test]
    fn test_set_rect_full() {
        let mut n = PNode::new(Region::new(0u32, 0, 2), false, false);
        n.draw_rect(&URect::new(0, 0, 2, 2), 1, true);
        assert!(n.value());
        assert!(n.is_leaf());
    }

    #[test]
    fn test_set_rect_contained() {
        let mut n = PNode::new(Region::new(0u32, 0, 2), false, false);
        n.draw_rect(&URect::new(0, 0, 1, 1), 1, true);
        assert!(!n.is_leaf());
        assert!(n.children()[Quadrant::BottomLeft as usize].value());
    }

    #[test]
    fn test_dirty() {
        let mut n = PNode::new(Region::new(0u32, 0, 2), false, false);
        assert!(!n.dirty);
        n.set_pixel((0, 0).into(), 1, true);
        assert!(n.dirty);
        assert!(n.children_mut()[Quadrant::BottomLeft as usize].dirty);
        assert!(!n.children_mut()[Quadrant::BottomRight as usize].dirty);
        assert!(!n.children_mut()[Quadrant::TopLeft as usize].dirty);
        assert!(!n.children_mut()[Quadrant::TopRight as usize].dirty);
    }

    #[test]
    fn test_drain_dirty_leaves() {
        let mut n = PNode::new(Region::new(0u32, 0, 2), false, false);
        n.set_pixel((0, 0).into(), 1, true);
        let mut traversed = 0;
        n.drain_dirty_leaves(
            &mut |node| {
                assert!(!node.dirty);
            },
            &mut traversed,
        );
        assert_eq!(traversed, 2);
        assert!(!n.children_mut()[Quadrant::BottomLeft as usize].dirty);
        assert!(!n.children_mut()[Quadrant::BottomRight as usize].dirty);
        assert!(!n.children_mut()[Quadrant::TopLeft as usize].dirty);
        assert!(!n.children_mut()[Quadrant::TopRight as usize].dirty);
    }

    #[test]
    fn test_visit_leaves_in_rect() {
        let n = PNode::new(Region::new(0u32, 0, 2), false, false);
        let mut traversed = 0;
        n.visit_leaves_in_rect(
            &URect::new(0, 0, 1, 2),
            &mut |node, sub_rect| {
                assert!(node.is_leaf());
                assert_eq!(sub_rect, &URect::new(0, 0, 1, 2));
            },
            &mut traversed,
        );
        assert_eq!(traversed, 1);
    }
}
