#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{
    Children, ICircle, IRect, PNode, RayCast, RayCastContext, RayCastQuery, RayCastResult, Region,
};
use crate::{ILine, NodePath, Quadrant, Shape};
use glam::IVec2;
use num_traits::{NumCast, Unsigned};
use std::collections::HashSet;
use std::fmt::{Debug, Formatter};

/// A map of pixels in a square region implemented by an MX quad tree.
/// The coordinate origin is at the bottom left of the region.
///
/// # Type Parameters
///
/// - `T`: The type of pixel data. By default a `bool`, to denote the pixel is on or off.
///   A more useful type could be a Color.
/// - `U`: The unsigned integer type of the coordinates used to index the pixels, typically `u16` (default), or `u32`.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq)]
pub struct PixelMap<T: Copy + PartialEq = bool, U: Unsigned + NumCast + Copy + Debug = u16> {
    root: PNode<T, U>,
    pixel_size: u8,
}

impl<T: Copy + PartialEq, U: Unsigned + NumCast + Copy + Debug> PixelMap<T, U> {
    /// Create a new [PixelMap].
    ///
    /// # Parameters
    ///
    /// - `region`: The region that this [PixelMap] covers.
    /// - `value`: The initial value of all pixels in this [PixelMap].
    /// - `pixel_size`: The pixel size of this [PixelMap] that is considered the smallest divisible unit.
    ///   Must be a power of two.
    ///
    /// # Panics
    ///
    /// If `pixel_size` is not a power of two.
    #[inline]
    #[must_use]
    pub fn new(region: Region<U>, value: T, pixel_size: u8) -> Self {
        assert!(region.size_as_usize().is_power_of_two());
        assert!(pixel_size.is_power_of_two());
        Self {
            root: PNode::new(region, value, true),
            pixel_size,
        }
    }

    /// Obtain the pixel size of this [PixelMap]. When a node's region is of this size, it cannot
    /// be subdivided further.
    #[inline]
    #[must_use]
    pub fn pixel_size(&self) -> u8 {
        self.pixel_size
    }

    /// Obtain the region that this [PixelMap] covers.
    #[inline]
    #[must_use]
    pub fn region(&self) -> &Region<U> {
        self.root.region()
    }

    /// Discard any existing pixel data and set the root node's value to that provided.
    ///
    /// # Parameters
    ///
    /// - `value`: The value to assign to the root node.
    #[inline]
    pub fn clear(&mut self, value: T) {
        self.root.set_value(value);
    }

    /// Get the value of the pixel at the given coordinates. If the coordinates are outside the
    /// region covered by this [PixelMap], None is returned.
    ///
    /// # Parameters
    ///
    /// - `point`: The coordinates of the pixel for which to retrieve the associated value.
    #[inline]
    #[must_use]
    pub fn get_pixel<P>(&self, point: P) -> Option<T>
    where
        P: Into<IVec2>,
    {
        let point = point.into();
        if self.root.region().contains(point) {
            Some(self.root.find_node(point).value())
        } else {
            None
        }
    }

    /// Get the path to the node that stores the pixel at the given point.
    ///
    /// # Parameters
    ///
    /// - `point`: The coordinates of the pixel for which to retrieve the node path.
    ///
    /// # Returns
    ///
    /// If the coordinates are outside the region covered by this [PixelMap], `None` is returned.
    #[inline]
    #[must_use]
    pub fn get_path<P>(&self, point: P) -> Option<NodePath>
    where
        P: Into<IVec2>,
    {
        let point = point.into();
        if self.root.region().contains(point) {
            let (_, path) = self.root.node_path(point);
            Some(path)
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
    /// If the coordinates are outside the region covered by this [PixelMap], `false` is returned.
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

    #[inline]
    pub fn draw(&mut self, shape: &Shape, value: T) -> bool {
        match shape {
            Shape::Point { point } => self.set_pixel(*point, value),
            Shape::Line { line } => self.draw_line(line, value),
            Shape::Rectangle { rect } => self.draw_rect(rect, value),
            Shape::Circle { circle } => self.draw_circle(circle, value),
        }
    }

    #[inline]
    pub fn draw_line(&mut self, line: &ILine, value: T) -> bool {
        if line.intersects_rect(&self.root.region().into()) {
            for p in line.pixels() {
                self.set_pixel(p, value);
            }
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
    /// the region covered by this [PixelMap], false is returned. Otherwise, true is returned.
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
    /// the region covered by this [PixelMap], false is returned. Otherwise, true is returned.
    #[inline]
    pub fn draw_circle(&mut self, circle: &ICircle, value: T) -> bool {
        if circle.aabb().intersects_rect(&self.root.region().into()) {
            self.root.draw_circle(circle, self.pixel_size, value);
            true
        } else {
            false
        }
    }

    /// Visit all leaf nodes in this [PixelMap] in pre-order.
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

    /// Visit all leaf nodes in this [PixelMap] that overlap with the given rectangle.
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
    /// `None` if `rect` does not overlap the region covered by this [PixelMap].
    #[inline]
    #[must_use]
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
    /// `None` if `rect` does not overlap the region covered by this [PixelMap].
    #[inline]
    #[must_use]
    pub fn all_in_rect<F>(&self, rect: &IRect, mut f: F) -> Option<bool>
    where
        F: FnMut(&PNode<T, U>, &IRect) -> bool,
    {
        self.root.all_leaves_in_rect(rect, &mut f)
    }

    /// Visit all leaf nodes in this [PixelMap] that are marked as dirty. This is useful for examining
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

    /// Visit dirty leaf nodes in this [PixelMap] that overlap with the given rectangle.
    /// This is useful for examining only leaf nodes that have changed (became dirty), and to
    /// limit time spent traversing the quad tree. Dirty status is not changed.
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
    pub fn visit_dirty_in_rect<F>(&self, rect: &IRect, mut visitor: F) -> usize
    where
        F: FnMut(&PNode<T, U>, &IRect),
    {
        let mut traversed = 0;
        self.root
            .visit_dirty_leaves_in_rect(rect, &mut visitor, &mut traversed);
        traversed
    }

    /// Visit all leaf nodes in this [PixelMap] that are marked as dirty, and consume
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

    /// Clear the dirty status of the root of this [PixelMap], according to a shallow or deep strategy.
    ///
    /// # Shallow Clear
    ///
    /// If dirty state is cleared in a shallow manner, the root node is marked clean, and
    /// the dirty state at any further depth is retained. Subsequent calls to other methods that
    /// navigate dirty nodes will not traverse any nodes, as none that are dirty are reachable
    /// (because the root node is no longer dirty).
    /// But, if branch `A` was dirty, [Self::clear_dirty] is called, and then branch `B` becomes dirty,
    /// both `A` and `B` will be traversed by [Self::visit_dirty()] or [Self::drain_dirty()].
    ///
    /// # Deep Clear
    ///
    /// A deep clear traverses all dirty nodes and marks them as clean.
    ///
    /// # Parameters
    ///
    /// - `deep`: If `true`, clear the dirty status of all nodes in this [PixelMap], otherwise
    ///   clear the dirty status of just the root node.
    #[inline]
    pub fn clear_dirty(&mut self, deep: bool) {
        if deep {
            self.drain_dirty(|_| {});
        } else {
            self.root.clear_dirty();
        }
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

    /// Obtain the points of node region corners that overlap with the given rectangle, and match
    /// the given predicate. Calls #[Self::collect_points] internally, but takes a guess at a
    /// reasonable capacity for the resulting HashSet.
    ///
    /// # Parameters
    ///
    /// - `rect`: The rectangle in which contained or overlapping nodes will be visited.
    /// - `offset`: An offset to apply to returned points.
    /// - `predicate`: A closure that takes a reference to a leaf node, and a reference to a rectangle as parameters.
    ///   This rectangle represents the intersection of the node's region and the `rect` parameter supplied to this method.
    ///   It returns `true` if the node matches the predicate, or `false` otherwise.
    pub fn points<F>(&self, rect: &IRect, offset: IVec2, predicate: F) -> HashSet<IVec2>
    where
        F: FnMut(&PNode<T, U>, &IRect) -> bool,
    {
        let mut result = HashSet::with_capacity(rect.area() as usize / 4);
        self.collect_points(rect, offset, predicate, &mut result);
        result
    }

    /// Collect the points of node region corners that overlap with the given rectangle, and match
    /// the given predicate.
    ///
    /// # Parameters
    ///
    /// - `rect`: The rectangle in which contained or overlapping nodes will be visited.
    /// - `offset`: An offset to apply to returned points.
    /// - `predicate`: A closure that takes a reference to a leaf node, and a reference to a rectangle as parameters.
    ///   This rectangle represents the intersection of the node's region and the `rect` parameter supplied to this method.
    ///   It returns `true` if the node matches the predicate, or `false` otherwise.
    /// - `hash`: A HashSet into which matching points will be inserted.
    #[inline]
    pub fn collect_points<F>(
        &self,
        rect: &IRect,
        offset: IVec2,
        mut predicate: F,
        hash: &mut HashSet<IVec2>,
    ) where
        F: FnMut(&PNode<T, U>, &IRect) -> bool,
    {
        self.visit_in_rect(rect, |node, sub_rect| {
            debug_assert!(!sub_rect.is_empty());
            if predicate(node, sub_rect) {
                let points = sub_rect.points();
                for p in points {
                    hash.insert(p + offset);
                }
            }
        });
    }

    /// Visit all leaf nodes in this [PixelMap] for which the region overlaps with the line
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
    #[inline]
    #[must_use]
    pub fn ray_cast<F>(&self, query: RayCastQuery, mut collision_check: F) -> RayCastResult
    where
        F: FnMut(&PNode<T, U>) -> RayCast,
    {
        let mut ctx = RayCastContext {
            line_iter: query.line.pixels(),
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

    /// Collect statistics by traversing the [PixelMap] quad tree.
    ///
    /// # Returns
    ///
    /// A [Stats] struct that contains information about [PixelMap]'s current state.
    #[must_use]
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

    /// Combine another [PixelMap] with this one using a function that decides how to combine
    /// the values of each pixel. This [PixelMap]'s region should overlap with the other [PixelMap]'s region,
    /// otherwise this operation has no effect.
    ///
    /// # Parameters
    ///
    /// - `other`: The other [PixelMap] to combine with this one.
    /// - `offset`: The other [PixelMap] is sampled according to this offset vector.
    /// - `combiner`: A closure that takes two values and returns a resulting value.
    ///
    /// # Examples
    ///
    /// This method can be used to implement boolean operations, such as union, intersection,
    /// disjunction, etc., according to the combiner function's implementation.
    ///
    /// To compute the union of two [PixelMap]s:
    /// ```
    /// # use pixel_map::{PixelMap, Region};
    /// # #[derive(Copy,Clone,PartialEq)]
    /// # enum Color { BLACK, WHITE }
    /// # let mut pixel_map: PixelMap<Color, u16> = PixelMap::new(Region::new(0, 0, 2), Color::WHITE, 1);
    /// # let mut other: PixelMap<Color, u16> = PixelMap::new(Region::new(0, 0, 2), Color::BLACK, 1);
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
    /// Compute an intersection of two [PixelMap]s:
    /// ```
    /// # use pixel_map::{PixelMap, Region};
    /// # #[derive(Copy,Clone,PartialEq)]
    /// # enum Color { BLACK, WHITE }
    /// # let mut pixel_map: PixelMap<Color, u16> = PixelMap::new(Region::new(0, 0, 2), Color::WHITE, 1);
    /// # let mut other: PixelMap<Color, u16> = PixelMap::new(Region::new(0, 0, 2), Color::BLACK, 1);
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

    /// Take the four top-level quadrant nodes in this [PixelMap] and
    /// create separate [PixelMap]s for each quadrant. The resulting slice can be indexed
    /// by [Quadrant].
    ///
    /// # Returns
    ///
    /// `Some` of a slice of four [PixelMap]s, one for each quadrant, if the top level node in this [PixelMap]
    /// has children. Otherwise, returns `None`.
    #[must_use]
    #[inline]
    pub fn split(mut self) -> Option<[PixelMap<T, U>; 4]> {
        match self.root.take_children() {
            Some(children) => {
                let result: [PixelMap<T, U>; 4] = children.map(|c| PixelMap {
                    root: c,
                    pixel_size: self.pixel_size,
                });
                Some(result)
            }
            None => None,
        }
    }

    /// Join the given four quadrant [PixelMap]s into a single [PixelMap].
    /// If any of the quadrant [PixelMap]s are dirty, the resulting [PixelMap] will be dirty.
    ///
    /// # Parameters
    ///
    /// - `quads`: The four quadrant [PixelMap]s to join. The regions of the four quadrant [PixelMap]s must
    ///   be the same size and must be offset such that they meet each other with no gaps or overlap.
    ///
    /// # Returns
    ///
    /// A new [PixelMap] that contains the four quadrant [PixelMap]s.
    ///
    /// # Panics
    ///
    /// - If the four quadrant [PixelMap]s have different pixel sizes.
    /// - If the four quadrant [PixelMap]s are different sizes.
    /// - If the four quadrant [PixelMap]s are not positioned in the `quads` slice according to [Quadrant].
    #[must_use]
    #[inline]
    pub fn join(quads: [PixelMap<T, U>; 4]) -> Self {
        let mut size: Option<u32> = None;
        let mut pixel_size: Option<u8> = None;
        let mut dirty = false;
        for pm in &quads {
            let quad_size: u32 = num_traits::cast::cast(pm.root.region().size()).unwrap();
            if let Some(s) = size {
                assert_eq!(s, quad_size);
            } else {
                size = Some(quad_size);
            }
            if let Some(ps) = pixel_size {
                assert_eq!(ps, pm.pixel_size);
            } else {
                pixel_size = Some(pm.pixel_size);
            }
            dirty = dirty || pm.root.dirty();
        }

        let bl = quads[Quadrant::BottomLeft as usize].root.region();
        let br = quads[Quadrant::BottomRight as usize].root.region();
        let tl = quads[Quadrant::TopLeft as usize].root.region();
        let tr = quads[Quadrant::TopRight as usize].root.region();
        assert_eq!(bl.x() + bl.size(), br.x());
        assert_eq!(tl.x() + tl.size(), tr.x());
        assert_eq!(bl.y() + bl.size(), tl.y());
        assert_eq!(br.y() + br.size(), tr.y());

        let children: Children<T, U> = Box::new(quads.map(|pm| pm.root));
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    fn test_u_type_parameters() {
        let _ = PixelMap::new(Region::new(0u8, 0, 2), false, 1);
        let _ = PixelMap::new(Region::new(0u16, 0, 2), false, 1);
        let _ = PixelMap::new(Region::new(0u32, 0, 2), false, 1);
        let _ = PixelMap::new(Region::new(0u64, 0, 2), false, 1);
        let _ = PixelMap::new(Region::new(0u128, 0, 2), false, 1);
    }

    #[test]
    fn test_clear() {
        let mut pm = PixelMap::new(Region::new(0u32, 0, 2), 0, 1);
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

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialization() {
        let mut pm: PixelMap<bool, u32> = PixelMap::new(Region::new(0u32, 0, 2), false, 1);
        pm.set_pixel((0, 0), true);

        let pmstr = ron::to_string(&pm).unwrap();
        let pm2: PixelMap<bool, u32> = ron::from_str(&pmstr).unwrap();

        assert_eq!(pm.root, pm2.root);
        assert_eq!(pm.pixel_size, pm2.pixel_size);
        assert!(pm2.get_pixel((0, 0)).unwrap());
    }
}
