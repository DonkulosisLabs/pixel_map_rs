# Changelog

## Unreleased

* `PixelMap::collect_points` supports non-default hasher with point `HashSet`.
* Substitute `bevy_math` for `glam`, in order to use established `IRect`.
* `PixelMap::new` accepts a `UVec2` rather than a `Region` to specify the map size. This allows non-square 
  dimensions, and ensures the map space is always (0, 0) at the bottom left corner.
* Add `PixelMap::map_size` to store a logical size for the map that is not restrained by the requirements
  of the underlying node implementation, which requires dimensions that are square and powers of two.
* Refactor shapes to be represented in unsigned space.

## v0.2.0

* Add `PixelMap::collect_points`.

## v0.1.0

* Initial release
