use std::fmt::Debug;

use super::{
    Children, ICircle, IRect, IVec2, PNode, RayCast, RayCastContext, RayCastQuery, RayCastResult,
    Region,
};
use num_traits::{NumCast, Unsigned};

/// A map of pixels implemented by an MX quad tree.
/// The {T} type parameter is the type of the pixel data, by default a bool, to denote the pixel is on or off.
/// A more useful type could be a Color, in which different data could be stored in each color channel.
/// The {U} type parameter is the type of the coordinates used to index the pixels, typically u16 (default), u32, or u64.
#[derive(Clone, PartialEq)]
pub struct PixelMap<T: Copy + PartialEq = bool, U: Unsigned + NumCast + Copy + Debug = u16> {
    root: PNode<T, U>,
    pixel_size: u8,
}

impl<T: Copy + PartialEq, U: Unsigned + NumCast + Copy + Debug> PixelMap<T, U> {
    /// Create a new {Self}. The pixel size must be a power of two.
    pub fn new(region: Region<U>, value: T, pixel_size: u8) -> Self {
        assert!(pixel_size.is_power_of_two());
        Self {
            root: PNode::new(region, value, true),
            pixel_size,
        }
    }

    #[inline]
    fn take_root(self) -> PNode<T, U> {
        self.root
    }

    /// Obtain the pixel size of this {Self}. When a node's region is of this size, it cannot
    /// be subdivided further.
    #[inline]
    pub fn pixel_size(&self) -> u8 {
        self.pixel_size
    }

    /// Obtain the region that this {Self} covers.
    #[inline]
    pub fn region(&self) -> &Region<U> {
        self.root.region()
    }

    /// Discard any existing pixel data and set the root node's value to that provided.
    #[inline]
    pub fn clear(&mut self, value: T) {
        self.root.set_value(value);
    }

    /// Get the value of the pixel at the given coordinates. If the coordinates are outside the
    /// region covered by this {Self}, None is returned.
    #[inline]
    pub fn get_pixel<P>(&self, point: P) -> Option<T>
    where
        P: Into<IVec2>,
    {
        let point = point.into();
        if self.root.region().contains(point) {
            Some(self.root.find_node(point, &mut 0).value())
        } else {
            None
        }
    }

    /// Set the value of the pixel at the given coordinates. If the coordinates are outside the
    /// region covered by this {Self}, false is returned. Otherwise, true is returned.
    #[inline]
    pub fn set_pixel<P>(&mut self, point: P, value: T) -> bool
    where
        P: Into<IVec2>,
    {
        let point = point.into();
        if self.root.region().contains(point) {
            self.root.set_pixel(point, self.pixel_size, value);
            true
        } else {
            false
        }
    }

    /// Set the color of the pixels within the given rectangle. If the rectangle does not overlap
    /// the region covered by this {Self}, false is returned. Otherwise, true is returned.
    #[inline]
    pub fn draw_rect(&mut self, rect: &IRect, value: T) -> bool {
        if rect.intersects_rect(&self.root.region().into()) {
            self.root.draw_rect(rect, self.pixel_size, value);
            true
        } else {
            false
        }
    }

    /// Set the color of the pixels within the given circle. If the circle's aabb does not overlap
    /// the region covered by this {Self}, false is returned. Otherwise, true is returned.
    #[inline]
    pub fn draw_circle(&mut self, circle: &ICircle, value: T) -> bool {
        if circle.aabb().intersects_rect(&self.root.region().into()) {
            self.root.draw_circle(circle, self.pixel_size, value);
            true
        } else {
            false
        }
    }

    /// Visit all leaf nodes in this {Self} in pre-order.
    #[inline]
    pub fn visit<F>(&self, mut visitor: F)
    where
        F: FnMut(&PNode<T, U>),
    {
        self.root.visit_leaves(&mut visitor);
    }

    /// Visit all leaf nodes in this {Self} that overlap with the given rectangle.
    /// A node reference as well as a rectangle representing the intersection of the node's region and
    /// the given rectangle are passed to the visitor.
    /// Returns the number of nodes traversed.
    #[inline]
    pub fn visit_in_rect<F>(&self, rect: &IRect, mut visitor: F) -> usize
    where
        F: FnMut(&PNode<T, U>, &IRect),
    {
        let mut traversed = 0;
        self.root
            .visit_leaves_in_rect(rect, &mut visitor, &mut traversed);
        traversed
    }

    /// Returns Some(true) if any of the leaf nodes within the bounds of the given rectangle match the
    /// predicate. Returns Some(false) if no nodes within the rect match the predicate.
    /// Returns None if the rect does not overlap the region covered by this {Self}.
    /// Node visitation short-circuits upon the first match.
    #[inline]
    pub fn any_in_rect<F>(&self, rect: &IRect, mut f: F) -> Option<bool>
    where
        F: FnMut(&PNode<T, U>, &IRect) -> bool,
    {
        self.root.any_leaves_in_rect(rect, &mut f)
    }

    /// Returns Some(true) if all of the leaf nodes within the bounds of the given rectangle match the
    /// predicate. Returns Some(false) if not all nodes within the rect match the predicate.
    /// Returns None if the rect does not overlap the region covered by this {Self}.
    /// Node visitation short-circuits upon the first non-match.
    #[inline]
    pub fn all_in_rect<F>(&self, rect: &IRect, mut f: F) -> Option<bool>
    where
        F: FnMut(&PNode<T, U>, &IRect) -> bool,
    {
        self.root.all_leaves_in_rect(rect, &mut f)
    }

    /// Visit all leaf nodes in this {Self} that are marked as dirty. This is useful for examining
    /// only leaf nodes that have changed (became dirty), and to limit time spent traversing
    /// the quad tree.
    /// Returns the number of nodes traversed.
    #[inline]
    pub fn visit_dirty<F>(&self, mut visitor: F) -> usize
    where
        F: FnMut(&PNode<T, U>),
    {
        let mut traversed = 0;
        self.root.visit_dirty_leaves(&mut visitor, &mut traversed);
        traversed
    }

    /// Visit all leaf nodes in this {Self} that are marked as dirty, and consume
    /// their dirty status (by setting them to dirty=false). This is useful for operating
    /// only on leaf nodes that have changed (became dirty), and to limit time spent traversing
    /// the quad tree.
    /// Returns the number of nodes traversed.
    #[inline]
    pub fn drain_dirty<F>(&mut self, mut visitor: F) -> usize
    where
        F: FnMut(&PNode<T, U>),
    {
        let mut traversed = 0;
        self.root.drain_dirty_leaves(&mut visitor, &mut traversed);
        traversed
    }

    /// Clear the dirty status of the root of this {Self}. This is done in a shallow manner,
    /// such that dirty state at any further depth is retained. Subsequent calls to {Self::visit_dirty}
    /// or {Self::drain_dirty} will not traverse any nodes as none that are dirty are reachable.
    /// But, if branch A was dirty, {Self::clear_dirty} is called, and then branch B becomes dirty,
    /// both A and B will be traversed by {Self::visit_dirty} or {Self::drain_dirty}.
    /// If a deep clear is desired, use {Self::drain_dirty} with a no-op visitor function.
    #[inline]
    pub fn clear_dirty(&mut self) {
        self.root.clear_dirty();
    }

    pub fn triangle_mesh_in_rect<F>(
        &self,
        rect: &IRect,
        offset: IVec2,
        mut predicate: F,
    ) -> (Vec<IVec2>, Vec<u32>)
    where
        F: FnMut(&PNode<T, U>, &IRect) -> bool,
    {
        let mut vertices: Vec<IVec2> = Vec::with_capacity(1024);
        let mut indices: Vec<u32> = Vec::with_capacity(1024);

        self.visit_in_rect(rect, |node, sub_rect| {
            debug_assert!(!sub_rect.is_empty());
            if predicate(node, sub_rect) {
                sub_rect.append_mesh_data(&mut vertices, &mut indices, offset);
            }
        });

        (vertices, indices)
    }

    /// Visit all leaf nodes in this {Self} for which the region overlaps with the line
    /// defined by the {RayCastQuery}. A collision_check function is called for each overlapping
    /// node, and must determine if a node represents a collision or if the ray should continue.
    pub fn ray_cast<F>(&self, query: RayCastQuery, mut collision_check: F) -> RayCastResult
    where
        F: FnMut(&PNode<T, U>) -> RayCast,
    {
        let mut ctx = RayCastContext {
            line_iter: query.line.iter(),
            traversed: 0,
        };
        if let Some(result) = self.root.ray_cast(&query, &mut ctx, &mut collision_check) {
            return result;
        }
        RayCastResult {
            collision_point: None,
            distance: 0.0,
            traversed: ctx.traversed,
        }
    }

    /// Collect statistics by traversing the {Self} quad tree.
    pub fn stats(&self) -> Stats {
        let mut stats = Stats::default();
        self.root.visit_nodes(&mut |node| {
            stats.node_count += 1;
            if node.is_leaf() {
                stats.leaf_count += 1;

                if node.region().is_unit(self.pixel_size) {
                    stats.unit_count += 1;
                }
            }
        });
        stats
    }

    /// Combine another {Self} with this one using a function that decides how to combine
    /// the colors of each pixel. This {Self}'s region should overlap with the other {Self}'s region,
    /// otherwise this operation has no effect.
    /// The other {Self} is sampled according to the given offset.
    /// The combiner function is passed the color of the pixel in this {Self}
    /// and the color of the pixel in the other {Self} that must be evaluated to
    /// produce a resulting color.
    ///
    /// This method can be used to implement boolean operations, such as union, intersection,
    /// disjunction, etc., according to the combiner function's implementation.
    /// For example, to compute the union of two {Self}s:
    /// ```
    /// // Union (OR)
    /// pixel_map.combine(&other, (0, 0), |c1, c2| {
    ///     if c1 == &Color::BLACK || c2 == &Color::BLACK {
    ///         Color::BLACK
    ///     } else {
    ///         Color::WHITE
    ///     }
    /// });
    /// ```
    /// Or to compute an intersection of two {Self}s:
    /// ```
    /// // Intersection (AND)
    /// pixel_map.combine(&other, (0, 0), |c1, c2| {
    ///    if c1 == &Color::BLACK && c2 == &Color::BLACK {
    ///       Color::BLACK
    ///   } else {
    ///      Color::WHITE
    ///  }
    /// });
    /// ```
    pub fn combine<P, F>(&mut self, other: &Self, offset: P, combiner: F)
    where
        P: Into<IVec2>,
        F: Fn(&T, &T) -> T,
    {
        let offset = offset.into();
        let mut updates: Vec<(IRect, T)> = Vec::new();
        self.visit(|node| {
            let mut region_rect: IRect = node.region().into();
            region_rect = region_rect + offset;
            other.visit_in_rect(&region_rect, |other_node, sub_rect| {
                let value = combiner(&node.value(), &other_node.value());
                updates.push((sub_rect.clone() - offset, value));
            });
        });
        for (rect, color) in updates {
            self.draw_rect(&rect, color);
        }
    }

    /// Take the four top-level quadrant nodes in this {Self} and
    /// create separate {Self}s for each quadrant. The resulting slice can be indexed
    /// by {Quadrant}.
    /// Returns None if the top level node in this {Self} has no children.
    pub fn split(&mut self) -> Option<[PixelMap<T, U>; 4]> {
        match self.root.take_children() {
            Some(children) => {
                let result: [PixelMap<T, U>; 4] = children.map(|c| PixelMap {
                    root: *c,
                    pixel_size: self.pixel_size,
                });
                Some(result)
            }
            None => None,
        }
    }

    /// Join the given four quadrant {Self}s into a single {Self}.
    /// The regions of the four quadrant {Self}s must be the same size and
    /// must be offset such that they meet each other with no gaps or overlap.
    pub fn join(value: T, quads: [PixelMap<T, U>; 4]) -> Self {
        let mut pixel_size: Option<u8> = None;
        let mut dirty = false;
        for pm in &quads {
            if let Some(ps) = pixel_size {
                assert_eq!(ps, pm.pixel_size);
            } else {
                pixel_size = Some(pm.pixel_size);
            }
            dirty = dirty || pm.root.dirty();
        }

        let children: Children<T, U> = quads.map(|pm| Box::new(pm.root));
        let root = PNode::with_children(value, children, dirty);

        Self {
            root,
            pixel_size: pixel_size.unwrap(),
        }
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Stats {
    /// The number of nodes in the quad tree.
    pub node_count: usize,

    /// The number of leaf nodes in the quad tree.
    pub leaf_count: usize,

    /// The number of leaf nodes in the quad tree for which the region is a unit pixel size.
    /// The unit size is defined by the `pixel_size` parameter of the {PixelMap} constructor.
    pub unit_count: usize,
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn test_clear() {
        let mut pm = PixelMap::new(Region::new(0u32, 0u32, 2u32), 0, 1);
        pm.set_pixel((1, 1), 1);
        pm.clear(2);
        assert_eq!(pm.root.value(), 2);
        assert!(pm.root.children().is_none());
    }

    #[test]
    fn test_stats_with_root_node() {
        let pm = PixelMap::new(Region::new(0u32, 0, 2), false, 1);
        assert_eq!(
            pm.stats(),
            Stats {
                node_count: 1,
                leaf_count: 1,
                unit_count: 0,
            }
        );
    }

    #[test]
    fn test_stats_with_children() {
        let mut pm = PixelMap::new(Region::new(0u32, 0, 2), false, 1);
        pm.set_pixel((1, 1), true);
        assert_eq!(
            pm.stats(),
            Stats {
                node_count: 5,
                leaf_count: 4,
                unit_count: 4,
            }
        );
    }

    #[test]
    fn test_stats_with_grandchildren() {
        let mut pm = PixelMap::new(Region::new(0u32, 0, 4), false, 1);
        pm.draw_rect(&IRect::new(0, 0, 2, 2), true);
        pm.set_pixel((0, 0), false);
        assert_eq!(
            pm.stats(),
            Stats {
                node_count: 9,
                leaf_count: 7,
                unit_count: 4,
            }
        );
    }

    #[test]
    fn test_compile_pixel_map_u8() {
        let pm = PixelMap::new(Region::new(0u8, 0, 2), 0u8, 1);
        assert_eq!(pm.root.value(), 0);
    }

    #[test]
    fn test_compile_pixel_map_u16() {
        let pm = PixelMap::new(Region::new(0u16, 0, 2), 0u16, 1);
        assert_eq!(pm.root.value(), 0);
    }

    #[test]
    fn test_compile_pixel_map_u32() {
        let pm = PixelMap::new(Region::new(0u32, 0, 2), 0u32, 1);
        assert_eq!(pm.root.value(), 0);
    }

    #[test]
    fn test_compile_pixel_map_u64() {
        let pm = PixelMap::new(Region::new(0u64, 0, 2), 0u64, 1);
        assert_eq!(pm.root.value(), 0);
    }

    #[test]
    fn test_compile_pixel_map_usize() {
        let pm = PixelMap::new(Region::new(0usize, 0, 2), 0usize, 1);
        assert_eq!(pm.root.value(), 0);
    }

    #[test]
    fn test_split() {
        let mut pm = PixelMap::new(Region::new(0u32, 0, 2), 0, 1);
        pm.set_pixel((0, 0), 1);
        assert!(pm.root.children().is_some());
        let children = pm.split().unwrap();
        assert!(pm.root.children().is_none());
        assert_eq!(children[Quadrant::BottomLeft as usize].root.value(), 1);
        assert_eq!(children[Quadrant::BottomRight as usize].root.value(), 0);
        assert_eq!(children[Quadrant::TopLeft as usize].root.value(), 0);
        assert_eq!(children[Quadrant::TopRight as usize].root.value(), 0);
    }

    #[test]
    fn test_join() {
        let mut pm = PixelMap::new(Region::new(0u32, 0, 2), false, 1);
        let region1 = pm.root.region().clone();
        pm.set_pixel((0, 0), true);
        let children = pm.split().unwrap();

        let pm = PixelMap::join(false, children);
        let region2 = pm.root.region().clone();
        assert_eq!(pm.get_pixel((0, 0)), Some(true));
        assert_eq!(pm.get_pixel((0, 1)), Some(false));
        assert_eq!(pm.get_pixel((1, 0)), Some(false));
        assert_eq!(pm.get_pixel((1, 1)), Some(false));

        assert_eq!(region1, region2);
    }

    #[test]
    fn test_any_in_rect() {
        let mut pm = PixelMap::new(Region::new(0u32, 0, 2), false, 1);

        assert_eq!(
            pm.any_in_rect(&IRect::new(0, 0, 2, 2), |n, _| n.value()),
            Some(false)
        );
        assert_eq!(
            pm.any_in_rect(&IRect::new(2, 2, 4, 4), |n, _| n.value()),
            None
        );

        pm.set_pixel((0, 0), true);

        assert_eq!(
            pm.any_in_rect(&IRect::new(0, 0, 2, 2), |n, _| n.value()),
            Some(true)
        );
        assert_eq!(
            pm.any_in_rect(&IRect::new(0, 0, 2, 2), |n, _| !n.value()),
            Some(true)
        );
        assert_eq!(
            pm.any_in_rect(&IRect::new(0, 0, 1, 1), |n, _| n.value()),
            Some(true)
        );
        assert_eq!(
            pm.any_in_rect(&IRect::new(1, 1, 2, 2), |n, _| n.value()),
            Some(false)
        );
    }

    #[test]
    fn test_all_in_rect() {
        let mut pm = PixelMap::new(Region::new(0u32, 0, 2), false, 1);

        assert_eq!(
            pm.all_in_rect(&IRect::new(0, 0, 2, 2), |n, _| !n.value()),
            Some(true)
        );
        assert_eq!(
            pm.all_in_rect(&IRect::new(2, 2, 4, 4), |n, _| n.value()),
            None
        );

        pm.set_pixel((0, 0), true);

        assert_eq!(
            pm.all_in_rect(&IRect::new(0, 0, 2, 2), |n, _| n.value()),
            Some(false)
        );
        assert_eq!(
            pm.all_in_rect(&IRect::new(0, 0, 2, 2), |n, _| !n.value()),
            Some(false)
        );
        assert_eq!(
            pm.all_in_rect(&IRect::new(0, 0, 1, 1), |n, _| n.value()),
            Some(true)
        );
        assert_eq!(
            pm.all_in_rect(&IRect::new(1, 1, 2, 2), |n, _| n.value()),
            Some(false)
        );
    }
}
