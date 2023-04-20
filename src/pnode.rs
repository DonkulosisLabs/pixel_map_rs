use super::{
    ICircle, IRect, Quadrant, RayCast, RayCastContext, RayCastQuery, RayCastResult, Region,
};
use crate::distance_to;
use glam::IVec2;
use num_traits::{NumCast, Unsigned};
use std::fmt::Debug;

pub type Children<T, U> = [Box<PNode<T, U>>; 4];

/// A node of a [PixelMap] quad tree.
#[derive(Clone, Debug, PartialEq)]
pub struct PNode<T: Copy + PartialEq = bool, U: Unsigned + NumCast + Copy + Debug = u16> {
    region: Region<U>,
    value: T,
    children: Option<Children<T, U>>,
    dirty: bool,
}

impl<T: Copy + PartialEq, U: Unsigned + NumCast + Copy + Debug> PNode<T, U> {
    #[inline]
    pub(super) fn new(region: Region<U>, value: T, dirty: bool) -> Self {
        Self {
            region,
            value,
            children: None,
            dirty,
        }
    }

    #[inline]
    pub(super) fn with_children(value: T, children: Children<T, U>, dirty: bool) -> Self {
        let mut rect: IRect = children[0].region().into();
        for child in &children[1..] {
            rect = rect.union(&child.region().into());
        }
        assert_eq!(rect.width(), rect.height());
        let region = Region::new(
            num_traits::cast::cast(rect.x()).unwrap(),
            num_traits::cast::cast(rect.y()).unwrap(),
            num_traits::cast::cast(rect.width()).unwrap(),
        );

        Self {
            region,
            value,
            children: Some(children),
            dirty,
        }
    }

    /// Obtain the region represented by this node.
    #[inline]
    pub fn region(&self) -> &Region<U> {
        &self.region
    }

    /// Determine if this node is in a dirty state. This can be used to represent a
    /// modified node that needs to be manipulated in some way (i.e. written to an Image texture).
    #[inline]
    pub fn dirty(&self) -> bool {
        self.dirty
    }

    /// Set the dirty state of this node to false.
    #[inline]
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Obtain this node's value.
    #[inline]
    pub fn value(&self) -> T {
        self.value
    }

    /// Set the value of this node. If this node has children, they will be discarded.
    /// This marks the node as dirty.
    #[inline]
    pub(super) fn set_value(&mut self, value: T) {
        self.dirty = true;
        self.value = value;
        self.children = None;
    }

    /// Obtain an array of the children of this node. If this node has no children, None is returned.
    #[inline]
    pub fn children(&self) -> Option<&Children<T, U>> {
        self.children.as_ref()
    }

    /// Get the child node in the given quadrant. If this node has no children, `None` is returned.
    #[inline]
    pub fn child(&self, quadrant: Quadrant) -> Option<&PNode<T, U>> {
        match &self.children {
            Some(children) => children.get(quadrant as usize).map(|c| c.as_ref()),
            None => None,
        }
    }

    /// Get a mutable reference to the child node in the given quadrant. If this node has no children,
    /// `None` is returned.
    #[inline]
    pub fn child_mut(&mut self, quadrant: Quadrant) -> Option<&mut PNode<T, U>> {
        match &mut self.children {
            Some(children) => children.get_mut(quadrant as usize).map(|c| c.as_mut()),
            None => None,
        }
    }

    // Take the children of this node, making it a leaf node, having a value of whatever
    // was in effect at the time it was subdivided into child nodes. This marks the node as dirty.
    #[inline]
    pub(super) fn take_children(&mut self) -> Option<Children<T, U>> {
        self.dirty = true;
        self.children.take()
    }

    /// Determine if this node is a leaf node. Leaves don't have children.
    #[inline]
    pub fn is_leaf(&self) -> bool {
        self.children.is_none()
    }

    /// Determine if all immediate children of this node are leaf nodes.
    #[inline]
    pub fn is_leaf_parent(&self) -> bool {
        if let Some(children) = &self.children {
            children.iter().all(|c| c.is_leaf())
        } else {
            false
        }
    }

    // Visit all nodes.
    pub(super) fn visit_nodes<F>(&self, visitor: &mut F)
    where
        F: FnMut(&PNode<T, U>),
    {
        visitor(self);
        if let Some(children) = &self.children {
            for child in children {
                child.visit_nodes(visitor);
            }
        }
    }

    // Visit all nodes, mutably.
    pub(super) fn visit_nodes_mut<F>(&mut self, visitor: &mut F)
    where
        F: FnMut(&mut PNode<T, U>),
    {
        visitor(self);
        if let Some(children) = &mut self.children {
            for child in children {
                child.visit_nodes_mut(visitor);
            }
        }
    }

    // Visit all leaf nodes.
    pub(super) fn visit_leaves<F>(&self, visitor: &mut F)
    where
        F: FnMut(&PNode<T, U>),
    {
        match self.children {
            Some(ref children) => {
                for child in children {
                    child.visit_nodes(visitor);
                }
            }
            None => visitor(self),
        }
    }

    // Visit all leaf nodes within a given rectangle boundary.
    pub(super) fn visit_leaves_in_rect<F>(
        &self,
        rect: &IRect,
        visitor: &mut F,
        traversed: &mut usize,
    ) where
        F: FnMut(&PNode<T, U>, &IRect),
    {
        *traversed += 1;

        let my_rect: IRect = self.region().into();
        if let Some(sub_rect) = my_rect.intersection(rect) {
            match self.children {
                Some(ref children) => {
                    for child in children {
                        child.visit_leaves_in_rect(rect, visitor, traversed);
                    }
                }
                None => visitor(self, &sub_rect),
            }
        }
    }

    pub(super) fn any_leaves_in_rect<F>(&self, rect: &IRect, f: &mut F) -> Option<bool>
    where
        F: FnMut(&PNode<T, U>, &IRect) -> bool,
    {
        let my_rect: IRect = self.region().into();
        if let Some(sub_rect) = my_rect.intersection(rect) {
            match self.children {
                Some(ref children) => {
                    for child in children {
                        if let Some(true) = child.any_leaves_in_rect(rect, f) {
                            return Some(true);
                        }
                    }
                }
                None => {
                    if f(self, &sub_rect) {
                        return Some(true);
                    }
                }
            }
            return Some(false);
        }
        None
    }

    pub(super) fn all_leaves_in_rect<F>(&self, rect: &IRect, f: &mut F) -> Option<bool>
    where
        F: FnMut(&PNode<T, U>, &IRect) -> bool,
    {
        let my_rect: IRect = self.region().into();
        if let Some(sub_rect) = my_rect.intersection(rect) {
            match self.children {
                Some(ref children) => {
                    for child in children {
                        if let Some(false) = child.all_leaves_in_rect(rect, f) {
                            return Some(false);
                        }
                    }
                }
                None => {
                    if !f(self, &sub_rect) {
                        return Some(false);
                    }
                }
            }
            return Some(true);
        }
        None
    }

    pub(super) fn visit_dirty_leaves<F>(&self, visitor: &mut F, traversed: &mut usize)
    where
        F: FnMut(&PNode<T, U>),
    {
        *traversed += 1;

        if self.dirty() {
            match self.children {
                Some(ref children) => {
                    for child in children {
                        child.visit_dirty_leaves(visitor, traversed);
                    }
                }
                None => visitor(self),
            }
        }
    }

    pub(super) fn drain_dirty_leaves<F>(&mut self, visitor: &mut F, traversed: &mut usize)
    where
        F: FnMut(&PNode<T, U>),
    {
        *traversed += 1;

        if self.dirty() {
            self.clear_dirty();
            match self.children {
                Some(ref mut children) => {
                    for child in children {
                        child.drain_dirty_leaves(visitor, traversed);
                    }
                }
                None => visitor(self),
            }
        }
    }

    // Get the node that contains the given coordinates. The coordinates must be
    // known to be within the bounds of this node.
    pub(super) fn find_node(&self, point: IVec2, traversed: &mut usize) -> &PNode<T, U> {
        *traversed += 1;
        if let Some(children) = &self.children {
            let q = self.region.quadrant_for(point);
            children[q as usize].find_node(point, traversed)
        } else {
            self
        }
    }

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
                match self.children {
                    Some(ref children) => {
                        let q = self.region.quadrant_for(current_point);
                        let result = children[q as usize].ray_cast(query, ctx, visitor);
                        if result.is_some() {
                            return result;
                        }
                        continue;
                    }
                    None => {
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

    pub(super) fn set_pixel(&mut self, point: IVec2, pixel_size: u8, value: T) -> bool {
        if self.region.contains(point) {
            if self.region.is_unit(pixel_size) {
                self.set_value(value);
            } else {
                self.subdivide();
                let q = self.region.quadrant_for(point);
                self.children.as_mut().unwrap()[q as usize].set_pixel(point, pixel_size, value);
                self.decimate();
                self.recalc_dirty();
            }
            return true;
        }
        false
    }

    pub(super) fn draw_rect(&mut self, rect: &IRect, pixel_size: u8, value: T) {
        if self.contained_by_rect(rect) {
            self.set_value(value);
        } else if let Some(sub_rect) = rect.intersection(&self.region().into()) {
            if self.region.is_unit(pixel_size) {
                self.set_value(value);
            } else {
                self.subdivide();
                let children = self.children.as_mut().unwrap();
                children[0].draw_rect(&sub_rect, pixel_size, value);
                children[1].draw_rect(&sub_rect, pixel_size, value);
                children[2].draw_rect(&sub_rect, pixel_size, value);
                children[3].draw_rect(&sub_rect, pixel_size, value);
                self.decimate();
                self.recalc_dirty();
            }
        }
    }

    pub(super) fn draw_circle(&mut self, circle: &ICircle, pixel_size: u8, value: T) {
        let outer_aabb = circle.aabb();
        let inner_aabb = circle.inner_aabb();
        if self.contained_by_rect(&inner_aabb) {
            self.set_value(value);
        } else if let Some(outer_aabb) = outer_aabb.intersection(&self.region().into()) {
            self.draw_rect(&inner_aabb, pixel_size, value);
            for (x, y) in circle.clone().into_iter() {
                if !outer_aabb.contains((x, y)) {
                    continue;
                }
                if inner_aabb.contains((x, y)) {
                    continue;
                }
                self.set_pixel((x, y).into(), pixel_size, value);
            }
        }
    }

    #[inline]
    pub fn contained_by_rect(&self, rect: &IRect) -> bool {
        rect.contains(self.region.point()) && rect.contains(self.region.end_point())
    }

    fn subdivide(&mut self) {
        if self.children.is_some() {
            return;
        }

        let x = self.region.x();
        let y = self.region.y();
        let half_size = self.region.center();

        self.children = Some([
            Box::new(PNode::new(
                Region::new(x, y, half_size),
                self.value,
                self.dirty,
            )),
            Box::new(PNode::new(
                Region::new(x + half_size, y, half_size),
                self.value,
                self.dirty,
            )),
            Box::new(PNode::new(
                Region::new(x + half_size, y + half_size, half_size),
                self.value,
                self.dirty,
            )),
            Box::new(PNode::new(
                Region::new(x, y + half_size, half_size),
                self.value,
                self.dirty,
            )),
        ]);
    }

    fn decimate(&mut self) {
        if !self.is_leaf_parent() {
            return;
        }

        if let Some(children) = &self.children {
            let mut all_same = false;
            let mut c: Option<T> = None;

            for child in children.iter() {
                if let Some(color) = c {
                    if color != child.value {
                        all_same = false;
                        break;
                    }
                } else {
                    c = Some(child.value);
                    all_same = true;
                }
            }
            if all_same {
                self.set_value(c.unwrap());
            }
        }
    }

    fn recalc_dirty(&mut self) {
        if let Some(children) = &self.children {
            self.dirty = children.iter().any(|child| child.dirty);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_subdivide() {
        let mut n = PNode::new(Region::new(0u32, 0, 4), false, false);
        n.subdivide();
        let children = n.children.unwrap();
        assert_eq!(
            children[0].as_ref(),
            &PNode::new(Region::new(0u32, 0, 2), false, false)
        );
        assert_eq!(
            children[1].as_ref(),
            &PNode::new(Region::new(2u32, 0, 2), false, false)
        );
        assert_eq!(
            children[2].as_ref(),
            &PNode::new(Region::new(2u32, 2, 2), false, false)
        );
        assert_eq!(
            children[3].as_ref(),
            &PNode::new(Region::new(0u32, 2, 2), false, false)
        );
    }

    #[test]
    fn test_decimate_frees_children() {
        let mut n = PNode::new(Region::new(0u32, 0, 4), false, false);
        n.subdivide();
        assert!(n.children.is_some());
        n.decimate();
        assert!(n.children.is_none());
    }

    #[test]
    fn test_decimate_retains_children() {
        let mut n = PNode::new(Region::new(0u32, 0, 4), false, false);
        n.subdivide();
        n.children.as_mut().unwrap()[0].value = true;
        n.decimate();
        assert!(!n.value);
        assert!(n.children.is_some());
    }

    #[test]
    fn test_find_node() {
        let mut n = PNode::new(Region::new(0u32, 0, 2), false, false);
        n.subdivide();
        n.children.as_mut().unwrap()[0].value = true;
        assert!(n.find_node((0, 0).into(), &mut 0).value());
        assert!(!n.find_node((1, 0).into(), &mut 0).value());
        assert!(!n.find_node((0, 1).into(), &mut 0).value());
        assert!(!n.find_node((1, 1).into(), &mut 0).value());
    }

    #[test]
    fn test_set_pixel_subdivides() {
        let mut n = PNode::new(Region::new(0u32, 0, 2), false, false);
        n.set_pixel((0, 0).into(), 1, true);
        assert!(!n.value);
        assert!(n.children.is_some());
        assert!(n.find_node((0, 0).into(), &mut 0).value());
        assert!(!n.find_node((1, 0).into(), &mut 0).value());
        assert!(!n.find_node((0, 1).into(), &mut 0).value());
        assert!(!n.find_node((1, 1).into(), &mut 0).value());
    }

    #[test]
    fn test_set_pixel_on_decimates() {
        let mut n = PNode::new(Region::new(0u32, 0, 2), false, false);
        n.set_pixel((0, 0).into(), 1, true);
        n.set_pixel((1, 0).into(), 1, true);
        n.set_pixel((0, 1).into(), 1, true);
        n.set_pixel((1, 1).into(), 1, true);
        assert!(n.value);
        assert!(n.children.is_none());
    }

    #[test]
    fn test_set_pixel_off_decimates() {
        let mut n = PNode::new(Region::new(0u32, 0, 2), false, false);
        n.set_pixel((0, 0).into(), 1, true);
        n.set_pixel((0, 0).into(), 1, false);
        assert!(!n.value);
        assert!(n.children.is_none());
    }

    #[test]
    fn test_visit() {
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
        n.draw_rect(&IRect::new(0, 0, 2, 2), 1, true);
        assert!(n.value);
        assert!(n.children.is_none());
    }

    #[test]
    fn test_set_rect_contained() {
        let mut n = PNode::new(Region::new(0u32, 0, 2), false, false);
        n.draw_rect(&IRect::new(0, 0, 1, 1), 1, true);
        assert!(n.children.is_some());
        assert!(n.children.as_ref().unwrap()[Quadrant::BottomLeft as usize].value);
    }

    #[test]
    fn test_dirty() {
        let mut n = PNode::new(Region::new(0u32, 0, 2), false, false);
        assert!(!n.dirty);
        n.set_pixel((0, 0).into(), 1, true);
        assert!(n.dirty);
        assert!(n.children.as_mut().unwrap()[Quadrant::BottomLeft as usize].dirty);
        assert!(!n.children.as_mut().unwrap()[Quadrant::BottomRight as usize].dirty);
        assert!(!n.children.as_mut().unwrap()[Quadrant::TopLeft as usize].dirty);
        assert!(!n.children.as_mut().unwrap()[Quadrant::TopRight as usize].dirty);
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
        assert_eq!(traversed, 5);
        assert!(!n.children.as_mut().unwrap()[Quadrant::BottomLeft as usize].dirty);
        assert!(!n.children.as_mut().unwrap()[Quadrant::BottomRight as usize].dirty);
        assert!(!n.children.as_mut().unwrap()[Quadrant::TopLeft as usize].dirty);
        assert!(!n.children.as_mut().unwrap()[Quadrant::TopRight as usize].dirty);
    }

    #[test]
    fn test_visit_leaves_in_rect() {
        let n = PNode::new(Region::new(0u32, 0, 2), false, false);
        let mut traversed = 0;
        n.visit_leaves_in_rect(
            &IRect::new(0, 0, 1, 2),
            &mut |node, sub_rect| {
                assert!(node.is_leaf());
                assert_eq!(sub_rect, &IRect::new(0, 0, 1, 2));
            },
            &mut traversed,
        );
        assert_eq!(traversed, 1);
    }
}
