# The `pixel_map` Rust Crate

[![crates.io](https://img.shields.io/crates/v/pixel_map)](https://crates.io/crates/pixel_map)
[![docs](https://docs.rs/pixel_map/badge.svg)](https://docs.rs/pixel_map/)
![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)
[![build status](https://github.com/DonkulosisLabs/pixel_map_rs/actions/workflows/ci.yml/badge.svg)](https://github.com/donkulosislabs/pixel_map_rs/actions?query=workflow%3A%22ci%22)

A `PixelMap` stores pixel data for an image in a quad tree structure.

## Overview

A `PixelMap` is an MX quadtree implementation, occupies a region of two-dimensional space at the 
root node, and subdivides down to the pixel level. A type-generic pixel data value can be stored
for each pixel in the map, but the tree structure optimizes storage for regions of common values.
A pixel value must be `Copy + PartialEq`.

Project status: **alpha**. Releases may contain breaking changes.

## Usage

### Installation

Add the crate to your `Cargo.toml`:

```bash
cargo add pixel_map
```

### Creating a Pixel Map

```rust
use pixel_map::PixelMap;

// Example pixel data
struct MyColor(u8, u8, u8);

let mut pixel_map = PixelMap<MyColor>::new(
    Region::new(
        0,   // position x
        0,   // position y
        1024 // size
    ),
    MyColor(0, 0, 0), // initial value
    1,                // pixel size
);

```

### Drawing on the PixelMap

```rust
// Set a pixel
pixel_map.set_pixel((11, 12), MyColor(255, 0, 0));

// Draw a line
pixel_map.draw_line(&ILine::new((500, 500), (600, 400)), MyColor(0, 255, 0));

// Draw a rectangle
pixel_map.draw_rect(&IRect::from_corners((200, 200), (300, 300)), MyColor(0, 0, 255));

// Draw a circle
pixel_map.draw_circle(&ICircle::new((500, 500), 100), MyColor(0, 0, 255));
```

### Navigating the PixelMap

```rust
// Visit all leaf nodes
pixel_map.visit(|node| {
    println!("region: {:?}, value: {:?}", node.region(), node.value());
});

// Visit all leaf nodes that have been modified
pixel_map.visit_dirty(|node| {
    println!("region: {:?}, value: {:?}", node.region(), node.value());
});
pixel_map.clear_dirty(true /*recurse*/);
```

## Features

* Set individual pixel values, or draw primitive shapes:
  * Lines
  * Rectangles
  * Circles
* Perform boolean operations against two pixel maps (i.e. union, intersection, difference, xor).
* Tree nodes maintain a dirty state for efficiently traversing recently modified regions of the tree.

## Limitations

* Loading and saving pixel data into various image formats is outside the scope of this crate. But,
  the basic operations necessary to both populate pixel data, and traverse the quad tree structure
  are provided. So, this is achievable in encompassing or accompanying code, according to the needs
  of your use case.

## License

`pixel_map` is free and open source. All code in this repository is dual-licensed under 
either of the following, at your option:

* MIT License ([`LICENSE-MIT`](LICENSE-MIT) or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0 ([`LICENSE-APACHE`](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
