//! PixelMap
//! ========
//!
//! A `PixelMap` is a 2D grid of pixels, implemented by an MX variant of a quad tree.
//! Tree nodes subdivide down to a supplied pixel size.
//! Quad tree depth is not artificially limited as with other quad tree implementations,
//! but is defined by the division "distance" between the root node region and the pixel size.
//! A type-generic value is stored for each pixel, but storage is optimized for regions of
//! pixels having the same value (as per the function of a quad tree).

mod direction;
mod math;
mod node_path;
mod pixel_map;
mod pnode;
mod quadrant;
mod ray_cast;
mod region;
mod shapes;

pub use self::{
    direction::*, math::*, node_path::*, pixel_map::*, pnode::*, quadrant::*, ray_cast::*,
    region::*, shapes::*,
};
