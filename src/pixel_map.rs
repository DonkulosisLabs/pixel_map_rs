use super::{
    Children, ICircle, IRect, PNode, RayCast, RayCastContext, RayCastQuery, RayCastResult, Region,
};
use glam::IVec2;
use num_traits::{NumCast, Unsigned};
use std::fmt::{Debug, Formatter};

/// A map of pixels implemented by an MX quad tree.
///
/// # Type Parameters
///
/// - `T`: The type of pixel data. By default a `bool`, to denote the pixel is on or off.
///   A more useful type could be a Color.
/// - `U`: The unsigned integer type of the coordinates used to index the pixels, typically `u16` (default), or `u32`.
#[derive(Clone, PartialEq)]
pub struct PixelMap<T: Copy + PartialEq = bool, U: Unsigned + NumCast + Copy + Debug = u16> {
    root: PNode<T, U>,
    pixel_size: u8,
}

impl<T: Copy + PartialEq, U: Unsigned + NumCast + Copy + Debug> PixelMap<T, U> {
    /// Create a new [Self]. The pixel size must be a power of two.
    ///
    /// # Parameters
    ///
    /// - `region`: The region that this [Self] covers.
    /// - `value`: The initial value of all pixels in this [Self].
    /// - `pixel_size`: The pixel size of this [Self] that is considered the smallest divisible unit.
    pub fn new(region: Region<U>, value: T, pixel_size: u8) -> Self {
        assert!(pixel_size.is_power_of_two());
        Self {
            root: PNode::new(region, value, true),
            pixel_size,
        }
    }

    /// Obtain the pixel size of this [Self]. When a node's region is of this size, it cannot
    /// be subdivided further.
    #[inline]
    pub fn pixel_size(&self) -> u8 {
        self.pixel_size
    }

    /// Obtain the region that this [Self] covers.
    #[inline]
    pub fn region(&self) -> &Region<U> {
        self.root.region()
    }

    /// Discard any existing pixel data and set the root node's value to that provided.
    ///
    /// # Parameters:
    ///
    /// - `value`: The value to assign to the root node.
    #[inline]
    pub fn clear(&mut self, value: T) {
        self.root.set_value(value);
    }

    /// Get the value of the pixel at the given coordinates. If the coordinates are outside the
    /// region covered by this [Self], None is returned.
    ///
    /// # Parameters
    ///
    /// - `point`: The coordinates of the pixel for which to retrieve the associated value.
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

    /// Set the value of the pixel at the given coordinates.
    ///
    /// # Parameters
    ///
    /// - `point`: The coordinates of the pixel for which to set the associated value.
    ///
    /// # Returns
    ///
    /// If the coordinates are outside the region covered by this [Self], `false` is returned.
    /// Otherwise, `true` is returned.
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

    /// Set the color of the pixels within the given rectangle.
    ///
    /// # Parameters
    ///
    /// - `rect`: The rectangle in which pixels will be set to associated value.
    /// - `value`: The value to assign to the pixels within the given rectangle.
    ///
    /// # Returns
    ///
    /// If the rectangle does not overlap
    /// the region covered by this [Self], false is returned. Otherwise, true is returned.
    #[inline]
    pub fn draw_rect(&mut self, rect: &IRect, value: T) -> bool {
        if rect.intersects_rect(&self.root.region().into()) {
            self.root.draw_rect(rect, self.pixel_size, value);
            true
        } else {
            false
        }
    }

    /// Set the color of the pixels within the given circle.
    ///
    /// # Parameters
    ///
    /// - `circle`: The circle in which pixels will be set to associated value.
    /// - `value`: The value to assign to the pixels within the given circle.
    ///
    /// # Returns
    ///
    /// If the circle's aabb does not overlap
    /// the region covered by this [Self], false is returned. Otherwise, true is returned.
    #[inline]
    pub fn draw_circle(&mut self, circle: &ICircle, value: T) -> bool {
        if circle.aabb().intersects_rect(&self.root.region().into()) {
            self.root.draw_circle(circle, self.pixel_size, value);
            true
        } else {
            false
        }
    }

    /// Visit all leaf nodes in this [Self] in pre-order.
    ///
    /// # Parameters
    ///
    /// - `visitor`: A closure that takes a reference to a leaf node as its only parameter.
    #[inline]
    pub fn visit<F>(&self, mut visitor: F)
    where
        F: FnMut(&PNode<T, U>),
    {
        self.root.visit_leaves(&mut visitor);
    }

    /// Visit all leaf nodes in this [Self] that overlap with the given rectangle.
    ///
    /// # Parameters
    ///
    /// - `rect`: The rectangle in which contained or overlapping nodes will be visited.
    /// - `visitor`: A closure that takes a reference to a leaf node, and a reference to a rectangle as parameters.
    ///   This rectangle represents the intersection of the node's region and the `rect` parameter supplied to this method.
    ///
    /// # Returns
    ///
    /// The number of nodes traversed.
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

    /// Determine if any of the leaf nodes within the bounds of the given rectangle match the predicate.
    /// Node visitation short-circuits upon the first match.
    ///
    /// # Parameters
    ///
    /// - `rect`: The rectangle in which contained or overlapping nodes will be visited.
    /// - `f`: A closure that takes a reference to a leaf node, and a reference to a rectangle as parameters.
    ///   This rectangle represents the intersection of the node's region and the `rect` parameter supplied to this method.
    ///   It returns `true` if the node matches the predicate, or `false` otherwise.
    ///
    /// # Returns
    ///
    /// `Some(true)` if any of the leaf nodes within the bounds of `rect` match the
    /// predicate. Or `Some(false)` if no nodes within `rect` match the predicate.
    /// `None` if `rect` does not overlap the region covered by this [Self].
    #[inline]
    pub fn any_in_rect<F>(&self, rect: &IRect, mut f: F) -> Option<bool>
    where
        F: FnMut(&PNode<T, U>, &IRect) -> bool,
    {
        self.root.any_leaves_in_rect(rect, &mut f)
    }

    /// Determine if all of the leaf nodes within the bounds of the given rectangle match the predicate.
    /// Node visitation short-circuits upon the first match.
    ///
    /// # Parameters
    ///
    /// - `rect`: The rectangle in which contained or overlapping nodes will be visited.
    /// - `f`: A closure that takes a reference to a leaf node, and a reference to a rectangle as parameters.
    ///   This rectangle represents the intersection of the node's region and the `rect` parameter supplied to this method.
    ///   It returns `true` if the node matches the predicate, or `false` otherwise.
    ///
    /// # Returns
    ///
    /// `Some(true)` if all of the leaf nodes within the bounds of `rect` match the
    /// predicate. Or `Some(false)` if none or some of the nodes within `rect` match the predicate.
    /// `None` if `rect` does not overlap the region covered by this [Self].
    #[inline]
    pub fn all_in_rect<F>(&self, rect: &IRect, mut f: F) -> Option<bool>
    where
        F: FnMut(&PNode<T, U>, &IRect) -> bool,
    {
        self.root.all_leaves_in_rect(rect, &mut f)
    }

    /// Visit all leaf nodes in this [Self] that are marked as dirty. This is useful for examining
    /// only leaf nodes that have changed (became dirty), and to limit time spent traversing
    /// the quad tree. Dirty status is not changed.
    ///
    /// # Parameters
    ///
    /// - `visitor`: A closure that takes a reference to a leaf node as its only parameter.
    ///
    /// # Returns
    ///
    /// The number of nodes traversed.
    #[inline]
    pub fn visit_dirty<F>(&self, mut visitor: F) -> usize
    where
        F: FnMut(&PNode<T, U>),
    {
        let mut traversed = 0;
        self.root.visit_dirty_leaves(&mut visitor, &mut traversed);
        traversed
    }

    /// Visit all leaf nodes in this [Self] that are marked as dirty, and consume
    /// their dirty status (by modifying their dirty state to be `false`). This is useful for operating
    /// only on leaf nodes that have changed (became dirty), and to limit time spent traversing
    /// the quad tree.
    ///
    /// # Parameters
    ///
    /// - `visitor`: A closure that takes a reference to a leaf node as its only parameter.
    ///
    /// # Returns
    ///
    /// The number of nodes traversed.
    #[inline]
    pub fn drain_dirty<F>(&mut self, mut visitor: F) -> usize
    where
        F: FnMut(&PNode<T, U>),
    {
        let mut traversed = 0;
        self.root.drain_dirty_leaves(&mut visitor, &mut traversed);
        traversed
    }

    /// Clear the dirty status of the root of this [Self]. This is done in a shallow manner,
    /// such that dirty state at any further depth is retained. Subsequent calls to [Self::visit_dirty()]
    /// or [Self::drain_dirty()] will not traverse any nodes as none that are dirty are reachable.
    /// But, if branch A was dirty, [Self::clear_dirty()] is called, and then branch B becomes dirty,
    /// both A and B will be traversed by [Self::visit_dirty()] or [Self::drain_dirty()].
    /// If a deep clear is desired, use [Self::drain_dirty()] with a no-op visitor function.
    #[inline]
    pub fn clear_dirty(&mut self) {
        self.root.clear_dirty();
    }

    ///
    pub fn trimesh_in_rect<F>(
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
                sub_rect.append_trimesh_data(&mut vertices, &mut indices, offset);
            }
        });

        (vertices, indices)
    }

    ///
    pub fn polylines_in_rect<F>(
        &self,
        rect: &IRect,
        offset: IVec2,
        mut predicate: F,
    ) -> (Vec<IVec2>, Vec<[u32; 2]>)
    where
        F: FnMut(&PNode<T, U>, &IRect) -> bool,
    {
        let mut vertices: Vec<IVec2> = Vec::with_capacity(1024);
        let mut indices: Vec<[u32; 2]> = Vec::with_capacity(1024);

        self.visit_in_rect(rect, |node, sub_rect| {
            debug_assert!(!sub_rect.is_empty());
            if predicate(node, sub_rect) {
                sub_rect.append_polyline_data(&mut vertices, &mut indices, offset);
            }
        });

        (vertices, indices)
    }

    /// Visit all leaf nodes in this [Self] for which the region overlaps with the line
    /// defined by the [RayCastQuery].
    ///
    /// # Parameters
    ///
    /// - `query`: A [RayCastQuery] that defines the line to cast.
    /// - `collision_check`: A closure that takes a reference to a leaf node as its only parameter.
    ///   It returns a [RayCast] value that determines if the node represents a collision or if the
    ///   ray should continue.
    ///
    /// # Returns
    ///
    /// A [RayCastResult] that contains the collision result and related information.
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

    /// Collect statistics by traversing the [Self] quad tree.
    ///
    /// # Returns
    ///
    /// A [Stats] struct that contains information about [Self]'s current state.
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

    /// Combine another [Self] with this one using a function that decides how to combine
    /// the colors of each pixel. This [Self]'s region should overlap with the other [Self]'s region,
    /// otherwise this operation has no effect.
    ///
    /// # Parameters
    ///
    /// - `other`: The other [Self] to combine with this one.
    /// - `offset`: The other [Self] is sampled according to this offset vector.
    /// - `combiner`: A closure that takes two values and returns a resulting value.
    ///
    /// # Examples
    ///
    /// This method can be used to implement boolean operations, such as union, intersection,
    /// disjunction, etc., according to the combiner function's implementation.
    ///
    /// To compute the union of two [Self]s:
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
    ///
    /// Compute an intersection of two [Self]s:
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

    /// Take the four top-level quadrant nodes in this [Self] and
    /// create separate [Self]s for each quadrant. The resulting slice can be indexed
    /// by [crate::Quadrant].
    ///
    /// # Returns
    ///
    /// `Some` of a slice of four [Self]s, one for each quadrant, if the top level node in this [Self]
    /// has children. Otherwise, returns `None`.
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

    /// Join the given four quadrant [Self]s into a single [Self].
    /// If any of the quadrant [Self]s are dirty, the resulting [Self] will be dirty.
    ///
    /// # Parameters
    ///
    /// - `quads`: The four quadrant [Self]s to join. The regions of the four quadrant [Self]s must
    ///   be the same size and must be offset such that they meet each other with no gaps or overlap.
    ///
    /// # Returns
    ///
    /// A new [Self] that contains the four quadrant [Self]s.
    ///
    /// # Panics
    ///
    /// - If the four quadrant [Self]s have different pixel sizes.
    /// - If the four quadrant [Self]s are different sizes.
    /// - If the four quadrant [Self]s are not positioned in the `quads` slice according to [crate::Quadrant].
    pub fn join(quads: [PixelMap<T, U>; 4]) -> Self {
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

        // TODO: validate quadrant positioning in slice

        let children: Children<T, U> = quads.map(|pm| Box::new(pm.root));
        let root = PNode::with_children(children, dirty);

        Self {
            root,
            pixel_size: pixel_size.unwrap(),
        }
    }
}

impl<T: Copy + PartialEq, U: Unsigned + NumCast + Copy + Debug> Debug for PixelMap<T, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PixelMap")
            .field("pixel_size", &self.pixel_size)
            .finish()
    }
}

/// Stores statistics about a [PixelMap].
/// See [PixelMap::stats].
#[derive(Debug, Default, Eq, PartialEq)]
pub struct Stats {
    /// The number of nodes in the quad tree.
    pub node_count: usize,

    /// The number of leaf nodes in the quad tree.
    pub leaf_count: usize,

    /// The number of leaf nodes in the quad tree for which the region is a unit pixel size.
    /// The unit size is defined by the `pixel_size` parameter of the [PixelMap] constructor.
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

        let pm = PixelMap::join(children);
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
