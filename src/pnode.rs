#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

use super::{ICircle, RayCast, RayCastContext, RayCastQuery, RayCastResult, Region};
use crate::{exclusive_urect, to_cropped_urect, CellFill, NodePath, Quadrant};
use bevy_math::{URect, UVec2};
use num_traits::{NumCast, Unsigned};
use std::fmt::Debug;

pub type Children<T, U> = Box<[PNode<T, U>; 4]>;

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
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

/// A node of a [crate::PixelMap] quadtree.
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
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
        match &self.kind {
            PNodeKind::Leaf(_) => false,
            PNodeKind::Branch(children) => children.iter().all(|c| c.is_leaf()),
        }
    }

    /// Determine how the node is filled (i.e. how child nodes are stored) based on the
    /// given `predicate` closure.
    ///
    /// # Leaf Nodes
    ///
    /// A leaf node will be considered [CellFill::Full] if the predicate
    /// returns `true` for that node, otherwise [CellFill::Empty].
    ///
    /// # Branch Nodes
    ///
    /// A branch node will produce a [CellFill] that reflects the quadrant(s) that are leaf nodes
    /// and pass the predicate. In other words, any quadrants that are not represented by
    /// the returned [CellFill] are either a complex sub-tree of nodes, or do not pass the
    /// predicate.
    pub fn node_fill_profile<F>(&self, mut predicate: F) -> CellFill
    where
        F: FnMut(&PNode<T, U>) -> bool,
    {
        if self.is_leaf() {
            if predicate(self) {
                CellFill::Full
            } else {
                CellFill::Empty
            }
        } else {
            let children = self.children();

            let mut check_quadrant = |q: Quadrant| {
                let child = &children[q as usize];
                child.is_leaf() && predicate(child)
            };

            let (bl, br, tl, tr) = (
                check_quadrant(Quadrant::BottomLeft),
                check_quadrant(Quadrant::BottomRight),
                check_quadrant(Quadrant::TopLeft),
                check_quadrant(Quadrant::TopRight),
            );

            match (bl, br, tl, tr) {
                (true, true, true, true) => CellFill::Full,
                (false, false, false, false) => CellFill::Empty,
                (true, false, false, false) => CellFill::BottomLeft,
                (false, true, false, false) => CellFill::BottomRight,
                (false, false, true, false) => CellFill::TopLeft,
                (false, false, false, true) => CellFill::TopRight,
                (true, true, false, false) => CellFill::Bottom,
                (false, false, true, true) => CellFill::Top,
                (true, false, true, false) => CellFill::Left,
                (false, true, false, true) => CellFill::Right,
                (true, false, false, true) => CellFill::BottomLeftTopRight,
                (false, true, true, false) => CellFill::BottomRightTopLeft,
                (false, true, true, true) => CellFill::NotBottomLeft,
                (true, false, true, true) => CellFill::NotBottomRight,
                (true, true, false, true) => CellFill::NotTopLeft,
                (true, true, true, false) => CellFill::NotTopRight,
            }
        }
    }

    /// If a rectangle can contour the given `fill` pattern without gaps, return that rectangle
    /// representation for this node's region. Otherwise, return `None`.
    pub fn node_fill_rect(&self, fill: CellFill) -> Option<URect> {
        if let Some(q) = fill.quadrant() {
            return Some(self.children()[q as usize].region().into());
        }
        match fill {
            CellFill::Full => Some(self.region().into()),
            CellFill::Bottom => {
                let children = self.children();
                let bottom_left = children[Quadrant::BottomLeft as usize].region().as_urect();
                let bottom_right = children[Quadrant::BottomRight as usize].region().as_urect();
                Some(URect::from_corners(bottom_left.min, bottom_right.max))
            }
            CellFill::Top => {
                let children = self.children();
                let top_left = children[Quadrant::TopLeft as usize].region().as_urect();
                let top_right = children[Quadrant::TopRight as usize].region().as_urect();
                Some(URect::from_corners(top_left.min, top_right.max))
            }
            CellFill::Left => {
                let children = self.children();
                let bottom_left = children[Quadrant::BottomLeft as usize].region().as_urect();
                let top_left = children[Quadrant::TopLeft as usize].region().as_urect();
                Some(URect::from_corners(bottom_left.min, top_left.max))
            }
            CellFill::Right => {
                let children = self.children();
                let bottom_right = children[Quadrant::BottomRight as usize].region().as_urect();
                let top_right = children[Quadrant::TopRight as usize].region().as_urect();
                Some(URect::from_corners(bottom_right.min, top_right.max))
            }
            _ => None,
        }
    }

    // Visit all nodes within the given rectangle boundary.
    pub(super) fn visit_nodes_in_rect<F>(&self, rect: &URect, visitor: &mut F, traversed: &mut u32)
    where
        F: FnMut(&PNode<T, U>, &URect) -> CellFill,
    {
        *traversed += 1;

        let sub_rect = self.region().intersect(rect);
        if !sub_rect.is_empty() {
            let node_profile = visitor(self, &sub_rect);
            if let PNodeKind::Branch(children) = &self.kind {
                let node_profile = node_profile as u8;
                for q in Quadrant::iter() {
                    if node_profile & q.as_bit() != 0 {
                        children[q as usize].visit_nodes_in_rect(rect, visitor, traversed);
                    }
                }
            }
        }
    }

    // Visit all leaf nodes within the given rectangle boundary.
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
                let q = node.region.quadrant_for_upoint(point);
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
                let q = node.region.quadrant_for_upoint(point);
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
            if self.region.contains_ipoint(current_point) {
                match self.kind {
                    PNodeKind::Branch(ref children) => {
                        let q = self.region.quadrant_for_ipoint(current_point);
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
                                let distance = query
                                    .line
                                    .start()
                                    .as_vec2()
                                    .distance(current_point.as_vec2());
                                let result = RayCastResult {
                                    collision_point: Some(current_point.as_uvec2()),
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
        if self.region.contains_upoint(point) {
            if self.is_leaf() && &value == self.value() {
                return true;
            }
            if self.region.is_unit(pixel_size) {
                self.set_value(value);
            } else {
                self.subdivide();
                let q = self.region.quadrant_for_upoint(point);
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
        let outer_rect = to_cropped_urect(&circle.aabb());
        let inner_rect = to_cropped_urect(&circle.inner_rect());
        if self.contained_by_rect(&inner_rect) {
            self.set_value(value);
        } else if !self.region().intersect(&outer_rect).is_empty() {
            self.draw_rect(&inner_rect, pixel_size, value);
            let inner_rect = exclusive_urect(&inner_rect);
            for p in circle.unsigned_pixels() {
                if inner_rect.contains(p) {
                    continue;
                }
                self.set_pixel(p, pixel_size, value);
            }
        }
    }

    pub(super) fn visit_neighbor_pairs_face<F>(&self, rect: &URect, visitor: &mut F)
    where
        F: FnMut(NeighborOrientation, &PNode<T, U>, &URect, &PNode<T, U>, &URect),
    {
        if let PNodeKind::Branch(ref children) = self.kind {
            let sub_rect = self.region().intersect(rect);
            if sub_rect.is_empty() {
                return;
            }

            for child in children.as_ref() {
                child.visit_neighbor_pairs_face(&sub_rect, visitor);
            }

            Self::visit_neighbor_pairs_edge_h(
                &children[Quadrant::BottomLeft as usize],
                &children[Quadrant::BottomRight as usize],
                rect,
                visitor,
            );
            Self::visit_neighbor_pairs_edge_h(
                &children[Quadrant::TopLeft as usize],
                &children[Quadrant::TopRight as usize],
                rect,
                visitor,
            );
            Self::visit_neighbor_pairs_edge_v(
                &children[Quadrant::BottomLeft as usize],
                &children[Quadrant::TopLeft as usize],
                rect,
                visitor,
            );
            Self::visit_neighbor_pairs_edge_v(
                &children[Quadrant::BottomRight as usize],
                &children[Quadrant::TopRight as usize],
                rect,
                visitor,
            );
        }
    }

    pub(super) fn visit_neighbor_pairs_edge_h<F>(
        left: &PNode<T, U>,
        right: &PNode<T, U>,
        rect: &URect,
        visitor: &mut F,
    ) where
        F: FnMut(NeighborOrientation, &PNode<T, U>, &URect, &PNode<T, U>, &URect),
    {
        match (left, right) {
            (left, right) if left.is_leaf() && right.is_leaf() => {
                let sub_rect_l = left.region().intersect(rect);
                let sub_rect_r = right.region().intersect(rect);
                visitor(
                    NeighborOrientation::Horizontal,
                    left,
                    &sub_rect_l,
                    right,
                    &sub_rect_r,
                );
            }
            (left, right) if left.is_leaf() && !right.is_leaf() => {
                Self::visit_neighbor_pairs_edge_h(
                    left,
                    &right.children()[Quadrant::BottomLeft as usize],
                    rect,
                    visitor,
                );
                Self::visit_neighbor_pairs_edge_h(
                    left,
                    &right.children()[Quadrant::TopLeft as usize],
                    rect,
                    visitor,
                );
            }
            (left, right) if !left.is_leaf() && right.is_leaf() => {
                Self::visit_neighbor_pairs_edge_h(
                    &left.children()[Quadrant::BottomRight as usize],
                    right,
                    rect,
                    visitor,
                );
                Self::visit_neighbor_pairs_edge_h(
                    &left.children()[Quadrant::TopRight as usize],
                    right,
                    rect,
                    visitor,
                );
            }
            (left, right) => {
                Self::visit_neighbor_pairs_edge_h(
                    &left.children()[Quadrant::BottomRight as usize],
                    &right.children()[Quadrant::BottomLeft as usize],
                    rect,
                    visitor,
                );
                Self::visit_neighbor_pairs_edge_h(
                    &left.children()[Quadrant::TopRight as usize],
                    &right.children()[Quadrant::TopLeft as usize],
                    rect,
                    visitor,
                );
            }
        }
    }

    pub(super) fn visit_neighbor_pairs_edge_v<F>(
        bottom: &PNode<T, U>,
        top: &PNode<T, U>,
        rect: &URect,
        visitor: &mut F,
    ) where
        F: FnMut(NeighborOrientation, &PNode<T, U>, &URect, &PNode<T, U>, &URect),
    {
        match (bottom, top) {
            (bottom, top) if bottom.is_leaf() && top.is_leaf() => {
                let sub_rect_b = bottom.region().intersect(rect);
                let sub_rect_t = top.region().intersect(rect);
                visitor(
                    NeighborOrientation::Vertical,
                    bottom,
                    &sub_rect_b,
                    top,
                    &sub_rect_t,
                );
            }
            (bottom, top) if bottom.is_leaf() && !top.is_leaf() => {
                Self::visit_neighbor_pairs_edge_v(
                    bottom,
                    &top.children()[Quadrant::BottomLeft as usize],
                    rect,
                    visitor,
                );
                Self::visit_neighbor_pairs_edge_v(
                    bottom,
                    &top.children()[Quadrant::BottomRight as usize],
                    rect,
                    visitor,
                );
            }
            (bottom, top) if !bottom.is_leaf() && top.is_leaf() => {
                Self::visit_neighbor_pairs_edge_v(
                    &bottom.children()[Quadrant::TopLeft as usize],
                    top,
                    rect,
                    visitor,
                );
                Self::visit_neighbor_pairs_edge_v(
                    &bottom.children()[Quadrant::TopRight as usize],
                    top,
                    rect,
                    visitor,
                );
            }
            (bottom, top) => {
                Self::visit_neighbor_pairs_edge_v(
                    &bottom.children()[Quadrant::TopLeft as usize],
                    &top.children()[Quadrant::BottomLeft as usize],
                    rect,
                    visitor,
                );
                Self::visit_neighbor_pairs_edge_v(
                    &bottom.children()[Quadrant::TopRight as usize],
                    &top.children()[Quadrant::BottomRight as usize],
                    rect,
                    visitor,
                );
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

/// Describes the orientation of a pair of neighboring nodes.
#[derive(Debug, PartialEq)]
pub enum NeighborOrientation {
    Horizontal,
    Vertical,
}

#[cfg(test)]
mod test {
    use super::*;

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
        n.visit_nodes_in_rect(
            &n.region().into(),
            &mut |_n, _r| {
                count += 1;
                CellFill::Full
            },
            &mut 0,
        );
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

    #[test]
    fn test_visit_neighbor_pairs_face() {
        let mut n = PNode::new(Region::new(0u32, 0, 2), false, false);
        n.set_pixel((0, 0).into(), 1, true); // Cause subdivision

        let mut calls: Vec<(NeighborOrientation, URect, URect, URect, URect)> = Vec::new();

        n.visit_neighbor_pairs_face(
            &n.region().into(),
            &mut |orientation, left, left_rect, right, right_rect| {
                calls.push((
                    orientation,
                    left.region.as_urect(),
                    *left_rect,
                    right.region.as_urect(),
                    *right_rect,
                ));
            },
        );

        assert_eq!(calls.len(), 4);

        assert_eq!(calls[0].0, NeighborOrientation::Horizontal);
        assert_eq!(calls[0].1, URect::new(0, 0, 1, 1));
        assert_eq!(calls[0].2, URect::new(0, 0, 1, 1));
        assert_eq!(calls[0].3, URect::new(1, 0, 2, 1));
        assert_eq!(calls[0].4, URect::new(1, 0, 2, 1));

        assert_eq!(calls[1].0, NeighborOrientation::Horizontal);
        assert_eq!(calls[1].1, URect::new(0, 1, 1, 2));
        assert_eq!(calls[1].2, URect::new(0, 1, 1, 2));
        assert_eq!(calls[1].3, URect::new(1, 1, 2, 2));
        assert_eq!(calls[1].4, URect::new(1, 1, 2, 2));

        assert_eq!(calls[2].0, NeighborOrientation::Vertical);
        assert_eq!(calls[2].1, URect::new(0, 0, 1, 1));
        assert_eq!(calls[2].2, URect::new(0, 0, 1, 1));
        assert_eq!(calls[2].3, URect::new(0, 1, 1, 2));
        assert_eq!(calls[2].4, URect::new(0, 1, 1, 2));

        assert_eq!(calls[3].0, NeighborOrientation::Vertical);
        assert_eq!(calls[3].1, URect::new(1, 0, 2, 1));
        assert_eq!(calls[3].2, URect::new(1, 0, 2, 1));
        assert_eq!(calls[3].3, URect::new(1, 1, 2, 2));
        assert_eq!(calls[3].4, URect::new(1, 1, 2, 2));
    }
}
