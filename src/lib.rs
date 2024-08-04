//! PixelMap
//! ========
//!
//! A `PixelMap` is a 2D grid of pixels, implemented by an MX variant of a quadtree.
//! Tree nodes subdivide down to a supplied pixel size.
//! Quadtree depth is not artificially limited as with other quadtree implementations,
//! but is defined by the division "distance" between the root node region and the pixel size.
//! A type-generic value is stored for each pixel, but storage is optimized for regions of
//! pixels having the same value (as per the function of a quadtree).

mod direction;
mod isocontour;
mod math;
mod node_path;
mod pixel_map;
mod pnode;
mod quadrant;
mod ray_cast;
mod region;
mod shapes;

pub use self::{
    direction::*, isocontour::*, math::*, node_path::*, pixel_map::*, pnode::*, quadrant::*,
    ray_cast::*, region::*, shapes::*,
};
