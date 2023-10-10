use crate::{iline, rect_points, ILine, LineStripPixelIterator, UnsignedPixelIterator};
use bevy_math::{ivec2, vec2, IRect, IVec2, Vec2};

/// A rectangle that is rotated around the center pivot point.
#[derive(Debug)]
pub struct RotatedIRect {
    /// The original axis-aligned rectangle.
    pub rect: IRect,

    /// The rotation to apply to the original rectangle in radians.
    pub rotation: f32,
}

impl RotatedIRect {
    /// Create a new rotated rectangle for the given axis-aligned rectangle and
    /// rotation value, in radians.
    #[inline]
    #[must_use]
    pub fn new(rect: IRect, rotation: f32) -> Self {
        Self { rect, rotation }
    }

    /// Obtain the four corners of the rotated rectangle.
    #[inline]
    #[must_use]
    pub fn rotated_points(&self) -> [Vec2; 4] {
        let center = self.rect.center().as_vec2();
        let cos_theta = self.rotation.cos();
        let sin_theta = self.rotation.sin();

        let mut corners = rect_points(&self.rect.as_rect());
        for corner in corners.iter_mut() {
            let x_rotated = corner.x - center.x;
            let y_rotated = corner.y - center.y;
            corner.x = cos_theta * x_rotated - sin_theta * y_rotated + center.x;
            corner.y = sin_theta * x_rotated + cos_theta * y_rotated + center.y;
        }
        corners
    }

    /// Obtain the four edge line segments of the rotated rectangle.
    #[inline]
    #[must_use]
    pub fn rotated_edges(&self) -> [ILine; 4] {
        let points = self.rotated_points();
        let (p1, p2, p3, p4) = (
            points[0].as_ivec2(),
            points[1].as_ivec2(),
            points[2].as_ivec2(),
            points[3].as_ivec2(),
        );

        [iline(p1, p2), iline(p2, p3), iline(p3, p4), iline(p4, p1)]
    }

    /// Calculate the axis-aligned bounding box of the rotated rectangle.
    #[inline]
    #[must_use]
    pub fn aabb(&self) -> IRect {
        let corners = self.rotated_points();

        let min_x = corners[0]
            .x
            .min(corners[1].x)
            .min(corners[2].x)
            .min(corners[3].x);
        let min_y = corners[0]
            .y
            .min(corners[1].y)
            .min(corners[2].y)
            .min(corners[3].y);
        let max_x = corners[0]
            .x
            .max(corners[1].x)
            .max(corners[2].x)
            .max(corners[3].x);
        let max_y = corners[0]
            .y
            .max(corners[1].y)
            .max(corners[2].y)
            .max(corners[3].y);

        IRect::from_corners(
            ivec2(min_x as i32, min_y as i32),
            ivec2(max_x as i32, max_y as i32),
        )
    }

    /// Calculate the largest inscribed rectangle within the rotated rectangle.
    #[inline]
    #[must_use]
    pub fn inner_rect(&self) -> IRect {
        let w = self.rect.width() as f32;
        let h = self.rect.height() as f32;
        if w <= 0.0 || h <= 0.0 {
            return IRect::new(0, 0, 0, 0);
        }

        let width_is_longer = w >= h;
        let (side_long, side_short) = if width_is_longer { (w, h) } else { (h, w) };

        let sin_a = self.rotation.sin().abs();
        let cos_a = self.rotation.cos().abs();

        let size = if side_short <= 2.0 * sin_a * cos_a * side_long || (sin_a - cos_a).abs() < 1e-10
        {
            let x = 0.5 * side_short;
            let (wr, hr) = if width_is_longer {
                (x / sin_a, x / cos_a)
            } else {
                (x / cos_a, x / sin_a)
            };
            vec2(wr, hr)
        } else {
            let cos_2a = cos_a * cos_a - sin_a * sin_a;
            let wr = (w * cos_a - h * sin_a) / cos_2a;
            let hr = (h * cos_a - w * sin_a) / cos_2a;
            vec2(wr, hr)
        };

        IRect::from_center_size(self.rect.center(), size.as_ivec2())
    }

    /// Iterate over the pixel coordinates of the rotated rectangle's edges.
    #[inline]
    #[must_use]
    pub fn edge_pixels(&self) -> LineStripPixelIterator {
        let mut points: Vec<IVec2> = self
            .rotated_points()
            .map(|p| p.as_ivec2())
            .into_iter()
            .collect();
        // Connect first and last points
        points.push(points[0]);
        LineStripPixelIterator::from_points(&points)
    }

    /// Iterate over the positive pixel coordinates of the rotated rectangle's edges.
    #[inline]
    #[must_use]
    pub fn unsigned_edge_pixels(&self) -> UnsignedPixelIterator<LineStripPixelIterator> {
        UnsignedPixelIterator::<LineStripPixelIterator>::new(self.edge_pixels())
    }

    /// Iterator over all pixel coordinates, inclusive, within the rotated rectangle.
    #[must_use]
    pub fn pixels(&self) -> LineStripPixelIterator {
        // Get all edge pixel coordinates
        let mut edge_pixels: Vec<IVec2> = self.edge_pixels().collect();

        // Sort edge pixel coordinates by y then x
        edge_pixels.sort_by(|a, b| {
            if a.y == b.y {
                a.x.cmp(&b.x)
            } else {
                a.y.cmp(&b.y)
            }
        });

        let mut rows: Vec<ILine> = Vec::with_capacity(edge_pixels.len() / 2);

        let mut row_points = 0;
        let mut cur_y = i32::MIN;
        let mut min_x = 0;
        let mut max_x = 0;
        for p in edge_pixels {
            if p.y == cur_y {
                // Continue current row
                max_x = p.x;
                row_points += 1;
            } else {
                // Start new row
                if row_points == 1 {
                    rows.push(iline(ivec2(min_x, cur_y), ivec2(min_x, cur_y)));
                } else if row_points > 1 {
                    rows.push(iline(ivec2(min_x, cur_y), ivec2(max_x, cur_y)));
                }
                cur_y = p.y;
                min_x = p.x;
                row_points = 1;
            }
        }

        // Complete final row
        if row_points == 1 {
            rows.push(iline(ivec2(min_x, cur_y), ivec2(min_x, cur_y)));
        } else if row_points > 1 {
            rows.push(iline(ivec2(min_x, cur_y), ivec2(max_x, cur_y)));
        }

        LineStripPixelIterator::from_lines(&rows)
    }

    /// Iterator over all positive pixel coordinates, inclusive, within the rotated rectangle.
    #[inline]
    #[must_use]
    pub fn unsigned_pixels(&self) -> UnsignedPixelIterator<LineStripPixelIterator> {
        UnsignedPixelIterator::<LineStripPixelIterator>::new(self.pixels())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotated_points() {
        let rect = IRect::new(0, 0, 10, 10);
        let rotated_rect = RotatedIRect::new(rect, 0.0);
        let points = rotated_rect.rotated_points();
        assert_eq!(points[0], vec2(0.0, 0.0));
        assert_eq!(points[1], vec2(10.0, 0.0));
        assert_eq!(points[2], vec2(10.0, 10.0));
        assert_eq!(points[3], vec2(0.0, 10.0));

        let rect = IRect::new(0, 0, 10, 10);
        let rotated_rect = RotatedIRect::new(rect, std::f32::consts::PI / 2.0);
        let points = rotated_rect.rotated_points();
        assert_eq!(points[0], vec2(10.0, 0.0));
        assert_eq!(points[1], vec2(10.0, 10.0));
        assert_eq!(points[2], vec2(0.0, 10.0));
        assert_eq!(points[3], vec2(0.0, 0.0));

        let rect = IRect::new(0, 0, 10, 10);
        let rotated_rect = RotatedIRect::new(rect, std::f32::consts::PI);
        let points = rotated_rect.rotated_points();
        assert_eq!(points[0], vec2(10.0, 10.0));
        assert_eq!(points[1], vec2(-4.7683716e-7, 10.0));
        assert_eq!(points[2], vec2(4.7683716e-7, -4.7683716e-7));
        assert_eq!(points[3], vec2(10.0, 4.7683716e-7));

        let rect = IRect::new(0, 0, 10, 10);
        let rotated_rect = RotatedIRect::new(rect, 3.0 * std::f32::consts::PI / 2.0);
        let points = rotated_rect.rotated_points();
        assert_eq!(points[0], vec2(0.0, 10.0));
        assert_eq!(points[1], vec2(0.0, 0.0));
        assert_eq!(points[2], vec2(10.0, 0.0));
        assert_eq!(points[3], vec2(10.0, 10.0));
    }

    #[test]
    fn test_pixels() {
        let rect = IRect::new(0, 0, 4, 4);
        let rotated_rect = RotatedIRect::new(rect, 0.);
        let pixels: Vec<IVec2> = rotated_rect.pixels().collect();

        assert_eq!(
            pixels,
            vec![
                ivec2(0, 0),
                ivec2(1, 0),
                ivec2(2, 0),
                ivec2(3, 0),
                ivec2(4, 0),
                ivec2(0, 1),
                ivec2(1, 1),
                ivec2(2, 1),
                ivec2(3, 1),
                ivec2(4, 1),
                ivec2(0, 2),
                ivec2(1, 2),
                ivec2(2, 2),
                ivec2(3, 2),
                ivec2(4, 2),
                ivec2(0, 3),
                ivec2(1, 3),
                ivec2(2, 3),
                ivec2(3, 3),
                ivec2(4, 3),
                ivec2(0, 4),
                ivec2(1, 4),
                ivec2(2, 4),
                ivec2(3, 4),
                ivec2(4, 4)
            ]
        );
    }
}
