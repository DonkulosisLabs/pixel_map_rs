# Changelog

## Unreleased

* Add support for drawing rotated rectangles.

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
