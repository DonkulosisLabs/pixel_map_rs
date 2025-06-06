# Changelog

## Unreleased

## v0.4.0

* Add support for drawing rotated rectangles.
* Add isocontour detection: `PixelMap::contour`. This method previously returned non-contiguous contouring line
  segments.
* Rename `PNodeFill` to `CellFill`.
* Add [Ramer-Douglas-Peucker](https://en.wikipedia.org/wiki/Ramer%E2%80%93Douglas%E2%80%93Peucker_algorithm) line
  simplification algorithm as `IsoLine::simplify`.
* Add `ILine::distance_to_point` and `ILine::distance_squared_to_point` functions.
* Add `PixelMap::non_uniform_quad_mesh` function.
* Add a series of neighbour navigation functions surrounding: `PixelMap::visit_neighbor`.
* Add A* grid pathfinding via `PixelMap::pathfind_a_star_grid`.
* Fix `ILine::intersects_rect` to detect lines fully encompassed by a rectangle.

## v0.3.0

* `PixelMap::collect_points` supports non-default hasher with point `HashSet`.
* Substitute `bevy_math` for `glam`, in order to use established `IRect`.
* `PixelMap::new` accepts a `UVec2` rather than a `Region` to specify the map size. This allows non-square
  dimensions, and ensures the map space is always (0, 0) at the bottom left corner.
* Add `PixelMap::map_size` to store a logical size for the map that is not restrained by the requirements
  of the underlying node implementation, which requires dimensions that are square and powers of two.
* Add `PixelMap::contour` method to obtain a list of line segments that contour a shape.
* Add `PNode::node_fill_profile(...) -> PNodeFill` for better understanding how a node is populated
  with pixel data.
* Allow more advanced tree navigation by producing a `PNodeFill` from a `PixelMap::visit_nodes_in_rect`
  visitor closure, rather than a `bool`, which controls child node visitation.
* Remove `Shape` enum.

## v0.2.0

* Add `PixelMap::collect_points`.

## v0.1.0

* Initial release
