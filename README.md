# The `pixel_map` Rust Crate

[![Crates.io](https://img.shields.io/crates/v/pixel_map)](https://crates.io/crates/pixel_map)
[![docs](https://docs.rs/pixel_map/badge.svg)](https://docs.rs/pixel_map/)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/DonkulosisLabs/pixel_map/blob/master/LICENSE)
[![Crates.io](https://img.shields.io/crates/d/pixel_map)](https://crates.io/crates/pixel_map)

A `pixel_map` stores pixel data for a square image in a quad tree structure.

## Features

* Stores data, like a colour value, for each pixel in the map, but
  optimizes storage for regions of common values (as per the function/purpose of quad trees).
* Set individual pixel values, or draw primitive shapes:
  * Rectangles
  * Circles
* Split a pixel map into owned quadrants for parallel processing, and merge quadrants 
  back into a unified pixel map.
* Perform boolean operations against two pixel maps (i.e. union, intersection, difference, xor).
* Tree nodes maintain a dirty state for efficiently traversing recently modified regions of the tree.

## Limitations

* Loading and saving pixel data into various image formats is outside the scope of this crate. But,
  the basic operations necessary to both populate pixel data, and traverse the quad tree structure
  are provided. So, this is achievable in encompassing or accompanying code, according to the needs
  of your use case.
* In order to simplify and optimise pixel map operations, the pixel map is always square, and the
  number of pixels in each dimension must be a power of two.


