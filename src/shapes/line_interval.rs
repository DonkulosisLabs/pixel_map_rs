#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::ILine;
use bevy_math::IVec2;

// Adapted from: https://github.com/ucarion/line_intersection

/*
License (MIT)

Copyright (c) 2017 Ulysse Carion

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
 */

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineInterval {
    line: ILine,
    interval_of_intersection: (f32, f32),
}

impl LineInterval {
    #[inline]
    #[must_use]
    pub fn line_segment(line: ILine) -> LineInterval {
        LineInterval {
            line,
            interval_of_intersection: (0.0, 1.0),
        }
    }

    #[inline]
    #[must_use]
    pub fn ray(line: ILine) -> LineInterval {
        LineInterval {
            line,
            interval_of_intersection: (0.0, f32::INFINITY),
        }
    }

    #[inline]
    #[must_use]
    pub fn line(line: ILine) -> LineInterval {
        LineInterval {
            line,
            interval_of_intersection: (f32::NEG_INFINITY, f32::INFINITY),
        }
    }

    /// Get the relationship between this line segment and another.
    #[must_use]
    pub fn relate(&self, other: &LineInterval) -> LineRelation {
        // see https://stackoverflow.com/a/565282
        let p = self.line.start();
        let q = other.line.start();
        let r = self.line.end() - self.line.start();
        let s = other.line.end() - other.line.start();

        let r_cross_s = Self::cross(&r, (s.x as f32, s.y as f32));
        let q_minus_p = q - p;
        let q_minus_p_cross_r = Self::cross(&q_minus_p, (r.x as f32, r.y as f32));

        // are the lines are parallel?
        if r_cross_s == 0.0 {
            // are the lines collinear?
            if q_minus_p_cross_r == 0.0 {
                // the lines are collinear
                LineRelation::Collinear
            } else {
                // the lines are parallel but not collinear
                LineRelation::Parallel
            }
        } else {
            // the lines are not parallel
            let t = Self::cross(&q_minus_p, Self::div(&s, r_cross_s));
            let u = Self::cross(&q_minus_p, Self::div(&r, r_cross_s));

            // are the intersection coordinates both in range?
            let t_in_range =
                self.interval_of_intersection.0 <= t && t <= self.interval_of_intersection.1;
            let u_in_range =
                other.interval_of_intersection.0 <= u && u <= other.interval_of_intersection.1;

            if t_in_range && u_in_range {
                // there is an intersection
                LineRelation::DivergentIntersecting(Self::t_coord_to_point(p, r, t))
            } else {
                // there is no intersection
                LineRelation::DivergentDisjoint
            }
        }
    }

    #[inline]
    #[must_use]
    fn cross(a: &IVec2, b: (f32, f32)) -> f32 {
        a.x as f32 * b.1 - a.y as f32 * b.0
    }

    #[inline]
    #[must_use]
    fn div(a: &IVec2, b: f32) -> (f32, f32) {
        (a.x as f32 / b, a.y as f32 / b)
    }

    #[inline]
    #[must_use]
    fn t_coord_to_point(p: IVec2, r: IVec2, t: f32) -> IVec2 {
        (p.x + (t * r.x as f32) as i32, p.y + (t * r.y as f32) as i32).into()
    }
}

#[derive(Debug, PartialEq)]
pub enum LineRelation {
    /// The line intervals are not parallel (or anti-parallel), and "meet" each other at exactly
    /// one point.
    DivergentIntersecting(IVec2),
    /// The line intervals are not parallel (or anti-parallel), and do not intersect; they "miss"
    /// each other.
    DivergentDisjoint,
    /// The line intervals lie on the same line. They may or may not overlap, and this intersection
    /// is possibly infinite.
    Collinear,
    /// The line intervals are parallel or anti-parallel.
    Parallel,
}

impl LineRelation {
    #[inline]
    #[must_use]
    pub fn unique_intersection(self) -> Option<IVec2> {
        match self {
            LineRelation::DivergentIntersecting(p) => Some(p),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divergent_intersecting_segments() {
        let a = ILine::new((100, 0), (100, 100));
        let b = ILine::new((0, 0), (200, 50));
        let s1 = LineInterval::line_segment(a);
        let s2 = LineInterval::line_segment(b);
        let relation = LineRelation::DivergentIntersecting((100, 25).into());

        assert_eq!(relation, s1.relate(&s2));
        assert_eq!(relation, s2.relate(&s1));
    }

    #[test]
    fn divergent_intersecting_segment_and_ray() {
        let a = ILine::new((0, 0), (100, 100));
        let b = ILine::new((200, 0), (200, 300));
        let s1 = LineInterval::ray(a);
        let s2 = LineInterval::line_segment(b);
        let relation = LineRelation::DivergentIntersecting((200, 200).into());

        assert_eq!(relation, s1.relate(&s2));
        assert_eq!(relation, s2.relate(&s1));
    }

    #[test]
    fn divergent_disjoint_segments() {
        let a = ILine::new((0, 0), (100, 100));
        let b = ILine::new((300, 0), (0, 300));
        let s1 = LineInterval::line_segment(a);
        let s2 = LineInterval::line_segment(b);
        let relation = LineRelation::DivergentDisjoint;

        assert_eq!(relation, s1.relate(&s2));
        assert_eq!(relation, s2.relate(&s1));
    }

    #[test]
    fn divergent_disjoint_ray_and_line() {
        let a = ILine::new((100, 100), (0, 0));
        let b = ILine::new((300, 0), (0, 300));
        let s1 = LineInterval::ray(a);
        let s2 = LineInterval::line(b);
        let relation = LineRelation::DivergentDisjoint;

        assert_eq!(relation, s1.relate(&s2));
        assert_eq!(relation, s2.relate(&s1));
    }

    #[test]
    fn parallel_disjoint_segments() {
        let a = ILine::new((0, 0), (100, 100));
        let b = ILine::new((0, 100), (100, 200));
        let s1 = LineInterval::line(a);
        let s2 = LineInterval::line(b);
        let relation = LineRelation::Parallel;

        assert_eq!(relation, s1.relate(&s2));
        assert_eq!(relation, s2.relate(&s1));
    }

    #[test]
    fn collinear_overlapping_segment_and_line() {
        let a = ILine::new((0, 0), (0, 150));
        let b = ILine::new((0, 400), (0, 500));
        let s1 = LineInterval::line(a);
        let s2 = LineInterval::ray(b);
        let relation = LineRelation::Collinear;

        assert_eq!(relation, s1.relate(&s2));
        assert_eq!(relation, s2.relate(&s1));
    }
}
