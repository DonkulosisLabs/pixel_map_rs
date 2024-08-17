use crate::{Direction, NeighborOrientation, PNode, PixelMap, Region};
use bevy_math::{URect, UVec2};
use num_traits::{NumCast, Unsigned};
use std::fmt::Debug;

impl<T: Copy + PartialEq, U: Unsigned + NumCast + Copy + Debug> PixelMap<T, U> {
    #[inline]
    pub fn visit_all_neighbors<F, V>(
        &self,
        rect: &URect,
        node_region: &URect,
        mut predicate: F,
        mut visitor: V,
    ) where
        F: FnMut(&PNode<T, U>, &URect) -> bool,
        V: FnMut(&PNode<T, U>, &URect),
    {
        Direction::iter()
            .for_each(|d| self.visit_neighbors(rect, node_region, d, &mut predicate, &mut visitor));
    }

    #[inline]
    pub fn visit_diagonal_neighbors<F, V>(
        &self,
        rect: &URect,
        node_region: &URect,
        mut predicate: F,
        mut visitor: V,
    ) where
        F: FnMut(&PNode<T, U>, &URect) -> bool,
        V: FnMut(&PNode<T, U>, &URect),
    {
        Direction::iter_diagonal()
            .for_each(|d| self.visit_neighbors(rect, node_region, d, &mut predicate, &mut visitor));
    }

    #[inline]
    pub fn visit_cardinal_neighbors<F, V>(
        &self,
        rect: &URect,
        node_region: &URect,
        mut predicate: F,
        mut visitor: V,
    ) where
        F: FnMut(&PNode<T, U>, &URect) -> bool,
        V: FnMut(&PNode<T, U>, &URect),
    {
        Direction::iter_cardinal()
            .for_each(|d| self.visit_neighbors(rect, node_region, d, &mut predicate, &mut visitor));
    }

    /// Visit neighboring nodes to the given node, on the specified edge or corner.  
    ///
    /// # Parameters
    ///
    /// - `rect`: The rectangle in which contained or overlapping nodes will be visited.
    /// - `node_region`: The region represented by the node for which to visit neighbors.
    /// - `direction`: The direction of the edge of the node for which to visit neighbors. When the
    ///   given direction is one of the diagonal variants, the single respective corner node
    ///   is visited.
    /// - `predicate`: A closure that takes a reference to a leaf node, and a reference to a rectangle as parameters.
    ///   This rectangle represents the intersection of the node's region and the `rect` parameter supplied to this method.
    ///   It returns `true` if the node matches the predicate, or `false` otherwise.
    /// - `visitor`: A closure that takes a reference to a leaf node, and a reference to a rectangle as parameters.
    ///   The node reference is a neighbor of the `node` passed to this method, on the edge of the given `direction`,
    ///   that has been accepted by the given `predicate` callback. This rectangle represents
    ///   the intersection of the node's region and the `rect` parameter supplied to this method.
    pub fn visit_neighbors<F, V>(
        &self,
        rect: &URect,
        node_region: &URect,
        direction: Direction,
        mut predicate: F,
        mut visitor: V,
    ) where
        F: FnMut(&PNode<T, U>, &URect) -> bool,
        V: FnMut(&PNode<T, U>, &URect),
    {
        let rect = rect.intersect(*node_region);

        let neighbor_rect = match rect_outer_edge(&rect, direction) {
            Some(r) => r,
            None => {
                return;
            }
        };

        self.root.visit_leaves_in_rect(
            &neighbor_rect,
            &mut |node, sub_rect| {
                if predicate(node, sub_rect) {
                    visitor(node, sub_rect);
                }
            },
            &mut 0,
        );
    }

    /// Visit all leaf nodes that intersect with the given `rect` that are neighbors.
    /// The `visitor` closure is called once for each unique pair of neighbor nodes.
    ///
    /// # Parameters
    ///
    /// - `rect`: The rectangle in which contained or overlapping nodes will be visited.
    /// - `visitor`: A closure that takes:
    ///   - A [NeighborOrientation] that indicates the orientation of the neighboring nodes.
    ///   - The left or bottom node, depending on the orientation.
    ///   - The rectangle that is the effective intersection of the left or bottom node's region
    ///     and the `rect` parameter supplied to this method.
    ///   - The right or top node, depending on the orientation.
    ///   - The rectangle that is the effective intersection of the right or top node's region
    ///     and the `rect` parameter supplied to this method.
    pub fn visit_neighbor_pairs<F>(&self, rect: &URect, visitor: &mut F)
    where
        F: FnMut(NeighborOrientation, &PNode<T, U>, &URect, &PNode<T, U>, &URect),
    {
        let sub_rect = self.map_rect.intersect(*rect);
        if !sub_rect.is_empty() {
            self.root.visit_neighbor_pairs_face(&sub_rect, visitor);
        }
    }
}

#[inline]
fn rect_outer_edge(rect: &URect, direction: Direction) -> Option<URect> {
    let edge = match direction {
        Direction::North => URect::from_corners(
            UVec2::new(rect.min.x, rect.max.y),
            UVec2::new(rect.max.x, rect.max.y + 1),
        ),
        Direction::NorthEast => URect::from_corners(rect.max, rect.max + 1),
        Direction::East => URect::from_corners(
            UVec2::new(rect.max.x, rect.min.y),
            UVec2::new(rect.max.x + 1, rect.max.y),
        ),
        Direction::SouthEast => {
            if rect.min.y == 0 {
                return None;
            }
            URect::from_corners(
                UVec2::new(rect.max.x, rect.min.y - 1),
                UVec2::new(rect.max.x + 1, rect.min.y),
            )
        }
        Direction::South => {
            if rect.min.y == 0 {
                return None;
            }
            URect::from_corners(
                UVec2::new(rect.min.x, rect.min.y - 1),
                UVec2::new(rect.max.x, rect.min.y),
            )
        }
        Direction::SouthWest => {
            if rect.min.x == 0 || rect.min.y == 0 {
                return None;
            }
            URect::from_corners(UVec2::new(rect.min.x - 1, rect.min.y - 1), rect.min)
        }
        Direction::West => {
            if rect.min.x == 0 {
                return None;
            }
            URect::from_corners(
                UVec2::new(rect.min.x - 1, rect.min.y),
                UVec2::new(rect.min.x, rect.max.y),
            )
        }
        Direction::NorthWest => {
            if rect.min.x == 0 {
                return None;
            }
            URect::from_corners(
                UVec2::new(rect.min.x - 1, rect.max.y),
                UVec2::new(rect.min.x, rect.max.y + 1),
            )
        }
    };

    Some(edge)
}

#[cfg(test)]
mod test {
    use crate::{Direction, PixelMap};
    use bevy_math::{uvec2, URect, UVec2};

    #[test]
    fn test_visit_neighbors_out_of_bounds() {
        let pm = PixelMap::<bool, u32>::new(&UVec2::splat(2), false, 1);
        pm.visit_all_neighbors(
            &pm.region().as_urect(),
            &pm.root.region().as_urect(),
            |n, _| *n.value(),
            |n, _| {
                assert!(false);
            },
        );

        pm.visit_all_neighbors(
            &pm.region().as_urect(),
            &pm.root.region().as_urect(),
            |n, _| !*n.value(),
            |n, _| {
                assert!(false);
            },
        );
    }

    #[test]
    fn test_visit_no_neighbors() {
        let mut pm = PixelMap::<u32, u16>::new(&UVec2::splat(4), 0, 1);
        pm.set_pixel(uvec2(1, 1), 10);

        let n = pm.root.find_node(uvec2(1, 1));
        pm.visit_all_neighbors(
            &pm.region().as_urect(),
            &n.region().as_urect(),
            |n, _| *n.value() != 0,
            |n, _| {
                assert!(false);
            },
        );
    }

    #[test]
    fn test_visit_neighbors() {
        let mut pm = PixelMap::<u32, u16>::new(&UVec2::splat(4), 0, 1);
        pm.set_pixel(uvec2(1, 1), 10); // Center
        pm.set_pixel(uvec2(1, 2), 20); // North
        pm.set_pixel(uvec2(2, 2), 30); // NorthEast
        pm.set_pixel(uvec2(2, 1), 40); // East
        pm.set_pixel(uvec2(2, 0), 50); // SouthEast
        pm.set_pixel(uvec2(1, 0), 60); // South
        pm.set_pixel(uvec2(0, 0), 70); // SouthWest
        pm.set_pixel(uvec2(0, 1), 80); // West
        pm.set_pixel(uvec2(0, 2), 90); // NorthWest

        let center = &pm.root.find_node(uvec2(1, 1)).region().as_urect();

        // North
        let mut visited = 0u32;
        pm.visit_neighbors(
            &pm.region().as_urect(),
            center,
            Direction::North,
            |n, _| *n.value() != 0,
            |n, _| {
                assert_eq!(
                    n.region().as_urect(),
                    URect::from_corners(uvec2(1, 2), uvec2(2, 3))
                );
                visited = *n.value()
            },
        );
        assert_eq!(visited, 20);

        // NorthEast
        let mut visited = 0u32;
        pm.visit_neighbors(
            &pm.region().as_urect(),
            center,
            Direction::NorthEast,
            |n, _| *n.value() != 0,
            |n, _| {
                assert_eq!(
                    n.region().as_urect(),
                    URect::from_corners(uvec2(2, 2), uvec2(3, 3))
                );
                visited = *n.value()
            },
        );
        assert_eq!(visited, 30);

        // East
        let mut visited = 0u32;
        pm.visit_neighbors(
            &pm.region().as_urect(),
            center,
            Direction::East,
            |n, _| *n.value() != 0,
            |n, _| {
                assert_eq!(
                    n.region().as_urect(),
                    URect::from_corners(uvec2(2, 1), uvec2(3, 2))
                );
                visited = *n.value()
            },
        );
        assert_eq!(visited, 40);

        // SouthEast
        let mut visited = 0u32;
        pm.visit_neighbors(
            &pm.region().as_urect(),
            center,
            Direction::SouthEast,
            |n, _| *n.value() != 0,
            |n, _| {
                assert_eq!(
                    n.region().as_urect(),
                    URect::from_corners(uvec2(2, 0), uvec2(3, 1))
                );
                visited = *n.value()
            },
        );
        assert_eq!(visited, 50);

        // South
        let mut visited = 0u32;
        pm.visit_neighbors(
            &pm.region().as_urect(),
            center,
            Direction::South,
            |n, _| *n.value() != 0,
            |n, _| {
                assert_eq!(
                    n.region().as_urect(),
                    URect::from_corners(uvec2(1, 0), uvec2(2, 1))
                );
                visited = *n.value()
            },
        );
        assert_eq!(visited, 60);

        // SouthWest
        let mut visited = 0u32;
        pm.visit_neighbors(
            &pm.region().as_urect(),
            center,
            Direction::SouthWest,
            |n, _| *n.value() != 0,
            |n, _| {
                assert_eq!(
                    n.region().as_urect(),
                    URect::from_corners(uvec2(0, 0), uvec2(1, 1))
                );
                visited = *n.value()
            },
        );
        assert_eq!(visited, 70);

        // West
        let mut visited = 0u32;
        pm.visit_neighbors(
            &pm.region().as_urect(),
            center,
            Direction::West,
            |n, _| *n.value() != 0,
            |n, _| {
                assert_eq!(
                    n.region().as_urect(),
                    URect::from_corners(uvec2(0, 1), uvec2(1, 2))
                );
                visited = *n.value()
            },
        );
        assert_eq!(visited, 80);

        // NorthWest
        let mut visited = 0u32;
        pm.visit_neighbors(
            &pm.region().as_urect(),
            center,
            Direction::NorthWest,
            |n, _| *n.value() != 0,
            |n, _| {
                assert_eq!(
                    n.region().as_urect(),
                    URect::from_corners(uvec2(0, 2), uvec2(1, 3))
                );
                visited = *n.value()
            },
        );
        assert_eq!(visited, 90);
    }
}
