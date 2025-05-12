use crate::nearest_neighbor::cell_neighbor;
use crate::{Direction, PNode, PixelMap};
use bevy_math::{uvec2, URect, UVec2};
use fxhash::FxHasher;
use indexmap::map::Entry::{Occupied, Vacant};
use indexmap::IndexMap;
use num_traits::{NumCast, Unsigned};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fmt::Debug;
use std::hash::BuildHasherDefault;

// Adapted from: https://github.com/evenfurther/pathfinding/blob/main/src/directed/astar.rs
// Released under a dual Apache 2.0 / MIT free software license.

/// Results from [PixelMap::pathfind_a_star_grid].
#[derive(Debug, Clone)]
pub struct PathfindAStarGridResult {
    /// A path of points from the desired `start` points to a `goal` point.
    /// Never empty.
    pub path: Vec<UVec2>,

    /// The cost value of the path.
    pub cost: u32,

    /// The number of cells examined to find the shortest path.
    pub considered_cells: u32,
}

type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;

impl<T: Copy + PartialEq, U: Unsigned + NumCast + Copy + Debug> PixelMap<T, U> {
    /// Find the shortest path from the `start` point to the `goal` point, using the
    /// A* algorithm to traverse a grid of cells over this quadtree. The grid, for which square
    /// cell size is defined by `cell_size`, is aligned with the `(0,0)` point
    /// (bottom-left of the quadtree) regardless of the given `bounds`. A path is determined by
    /// examining cells, and a cell is considered navigable when all nodes that compose the cell
    /// pass the given `predicate`. Apart from the `start` and `goal` points, resulting path
    /// points are positioned at the center of navigable cells.
    ///
    /// # Parameters
    ///
    /// - `bounds`: The rectangle in which contained or overlapping cells will be considered.
    /// - `cell_size`: The size of an edge, in pixels, of a single square cell in the grid to
    ///   navigate. A lower value will produce better path precision, but will take longer
    ///   to compute being that there will be more potential paths to consider. This value also
    ///   contributes to the effective spacing from walls that a resulting path will take. For
    ///   example, a `cell_size` of 50 would produce a path that is roughly 25 pixels away from
    ///   walls, minimum. Also consider that a gap between walls must be at least this
    ///   size in order to path through it. And, being that the grid is fixed to the zero point
    ///   a gap that is actually wider than the `cell_size` may still deny a path if the gap
    ///   straddles a border between two cells, where the `predicate` rejects both of those cells
    ///   due to overlap with either wall. So, generally, at least 2x the `cell_size` is
    ///   what can safely be considered the minimum allowable gap between walls that a path
    ///   can take.
    /// - `start`: The origin point of the potential path.
    /// - `goal`: The destination point of the potential path.
    /// - `heuristic`: The `A*` algorithm heuristic function. A general-purpose
    ///   [euclidean_heuristic] is provided. But, a heuristic tuned specifically for your use case
    ///   can produce significant performance improvements.
    /// - `predicate`: A closure that takes a reference to a leaf node, and a reference to a
    ///   rectangle as parameters. This rectangle represents the intersection of the node's
    ///   region and the `bounds` parameter supplied to this method. It returns `true` if the
    ///   node matches the predicate, or `false` otherwise. The `predicate` function is consulted
    ///   for every node the composes a cell (with short-circuit), and cell is considered to be
    ///   navigable only if all nodes produce a `true` result.
    ///
    /// # Returns
    ///
    /// `None is returned under the following conditions:
    ///
    /// - The `bounds` does not intersect with this quadtree's [PixelMap::map_rect].
    /// - The `start` or `goal` points do not fall within the intersection of the `bounds`
    ///   rectangle and this quadtree's [PixelMap::map_rect].
    /// - The nodes representing the `start` or `goal` points do not pass the `predicate`.
    /// - A navigable path is not possible.
    ///
    /// Otherwise, `Some` of a [PathfindAStarGridResult] is returned.
    pub fn pathfind_a_star_grid<H, F>(
        &self,
        bounds: &URect,
        cell_size: u32,
        start: UVec2,
        goal: UVec2,
        heuristic: H,
        mut predicate: F,
    ) -> Option<PathfindAStarGridResult>
    where
        H: Fn(&UVec2, &UVec2) -> u32,
        F: FnMut(&PNode<T, U>, &URect) -> bool,
    {
        if cell_size < 1 {
            panic!("grid_size must be >= 1");
        }
        let grid_half_size = cell_size / 2;

        let bounds = bounds.intersect(self.map_rect());
        if bounds.is_empty() {
            return None;
        }

        let start_node = self.root.find_node(start);

        // Special case: start or goal point is out of bounds
        if !bounds.contains(start) || !bounds.contains(goal) {
            return None;
        }

        // Special case: start node does not match predicate
        {
            let sub_rect = bounds.intersect(start_node.region().as_urect());
            if !predicate(start_node, &sub_rect) {
                return None;
            }
        }

        // Special case: goal node does not match predicate
        {
            let goal_node = self.root.find_node(goal);
            let sub_rect = bounds.intersect(goal_node.region().as_urect());
            if !predicate(goal_node, &sub_rect) {
                return None;
            }
        }

        // Special case: start and goal are within one node -> draw straight line
        if start_node.region().contains_upoint(goal) {
            let path = vec![start, goal];
            return Some(PathfindAStarGridResult {
                path,
                cost: 0,
                considered_cells: 1,
            });
        }

        let mut to_see = BinaryHeap::with_capacity(512);
        to_see.push(SmallestCostHolder {
            estimated_cost: 0,
            cost: 0,
            index: 0,
        });

        let mut parents: FxIndexMap<UVec2, (u32, u32)> = FxIndexMap::default();
        let start_cell = cell_for_point(start, cell_size);
        parents.insert(start_cell.min, (u32::MAX, 0));

        let mut considered_cells = 1;
        let mut direction_toggle = false;
        let mut last_successful_direction: Direction = Direction::North;

        while let Some(SmallestCostHolder { cost, index, .. }) = to_see.pop() {
            let cell = {
                let (cell_min, &(_, c)) = parents.get_index(index as usize).unwrap(); // Cannot fail
                let cell = URect::from_corners(*cell_min, *cell_min + cell_size);

                // Are we done?
                if cell.contains(goal) {
                    let path = reverse_path(parents, index);

                    // Map path points to cell centres
                    let mut path: Vec<UVec2> =
                        path.into_iter().map(|min| min + grid_half_size).collect();

                    // Replace first point (cell centre) with start
                    *path.get_mut(0).unwrap() = start;

                    // Replace last point (cell centre) with goal
                    let len = path.len();
                    *path.get_mut(len - 1).unwrap() = goal;

                    return Some(PathfindAStarGridResult {
                        path,
                        cost,
                        considered_cells,
                    });
                }
                if cost > c {
                    continue;
                }

                cell
            };

            direction_toggle = !direction_toggle;

            directions(last_successful_direction, direction_toggle)
                .into_iter()
                .for_each(|d| {
                    considered_cells += 1;

                    let neighbor_cell = cell_neighbor(&cell, d);
                    if neighbor_cell.is_empty() {
                        return;
                    }

                    match self.root.all_leaves_in_rect(&cell, &mut predicate) {
                        Some(pass) => {
                            if !pass {
                                return;
                            }
                        }
                        None => return,
                    };

                    let move_cost = 1; // TODO
                    let new_cost = cost + move_cost;
                    let h; // heuristic(&successor)
                    let i; // index for successor

                    match parents.entry(neighbor_cell.min) {
                        Vacant(e) => {
                            h = heuristic(&(*e.key() + grid_half_size), &goal);
                            i = e.index() as u32;
                            e.insert((index, new_cost));
                        }
                        Occupied(mut e) => {
                            if e.get().1 > new_cost {
                                h = heuristic(&(*e.key() + grid_half_size), &goal);
                                i = e.index() as u32;
                                e.insert((index, new_cost));
                            } else {
                                return;
                            }
                        }
                    }

                    last_successful_direction = d;
                    to_see.push(SmallestCostHolder {
                        estimated_cost: new_cost + h,
                        cost: new_cost,
                        index: i,
                    });
                });
        }
        None
    }
}

#[inline]
fn reverse_path(parents: FxIndexMap<UVec2, (u32, u32)>, start: u32) -> Vec<UVec2> {
    let mut i = start;
    let path = std::iter::from_fn(|| {
        parents.get_index(i as usize).map(|(node, value)| {
            i = value.0;
            *node
        })
    })
    .collect::<Vec<_>>();
    path.into_iter().rev().collect()
}

#[inline]
pub fn euclidean_heuristic(a: &UVec2, b: &UVec2) -> u32 {
    let dx = (a.x as f64 - b.x as f64).powi(2);
    let dy = (a.y as f64 - b.y as f64).powi(2);

    (dx + dy).abs() as u32
}

struct SmallestCostHolder<K> {
    estimated_cost: K,
    cost: K,
    index: u32,
}

impl<K: PartialEq> PartialEq for SmallestCostHolder<K> {
    fn eq(&self, other: &Self) -> bool {
        self.estimated_cost.eq(&other.estimated_cost) && self.cost.eq(&other.cost)
    }
}

impl<K: PartialEq> Eq for SmallestCostHolder<K> {}

impl<K: Ord> PartialOrd for SmallestCostHolder<K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: Ord> Ord for SmallestCostHolder<K> {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.estimated_cost.cmp(&self.estimated_cost) {
            Ordering::Equal => self.cost.cmp(&other.cost),
            s => s,
        }
    }
}

#[inline]
#[must_use]
pub fn cell_for_point(point: UVec2, grid_size: u32) -> URect {
    let min = uvec2(point.x / grid_size, point.y / grid_size) * grid_size;
    let max = uvec2(min.x + grid_size, min.y + grid_size) * grid_size;
    URect::from_corners(min, max)
}

fn directions(last_success: Direction, direction_toggle: bool) -> [Direction; 8] {
    let mut all = Direction::ALL;

    if direction_toggle {
        all.reverse();
    }

    if last_success != all[0] {
        let i = all.iter().position(|d| d == &last_success).unwrap();
        all.swap(0, i);
    }

    all
}
