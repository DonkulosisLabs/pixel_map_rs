use crate::{distance_to_line, ILine};
use bevy_math::IVec2;
use num_traits::Zero;
use std::collections::HashMap;

type FragmentKey = usize;

type Fragment = Vec<IVec2>;

pub(super) struct FragmentAccumulator {
    next_key: FragmentKey,
    fragments: HashMap<FragmentKey, Fragment>,
    by_start: HashMap<IVec2, FragmentKey>,
    by_end: HashMap<IVec2, FragmentKey>,
}

impl FragmentAccumulator {
    #[inline]
    pub(super) fn new(size: usize) -> Self {
        Self {
            next_key: 0,
            fragments: HashMap::with_capacity(size),
            by_start: HashMap::with_capacity(size),
            by_end: HashMap::with_capacity(size),
        }
    }

    #[inline]
    pub(super) fn result(self) -> Vec<IsoLine> {
        assert_eq!(self.by_start.len(), self.fragments.len());
        assert_eq!(self.by_end.len(), self.fragments.len());
        self.fragments
            .into_values()
            .map(|frag| IsoLine { points: frag })
            .collect()
    }

    #[inline]
    fn create_key(&mut self) -> FragmentKey {
        let key = self.next_key;
        self.next_key += 1;
        key
    }

    fn attach_fragment(&mut self, mut new_frag: Fragment) {
        if let Some(key) = self.by_end.remove(new_frag.first().unwrap()) {
            // [existing_frag_start, existing_frag_end] <-- [new_frag_start, new_frag_end]
            let mut existing_frag = self.fragments.remove(&key).unwrap();
            self.by_start.remove(existing_frag.first().unwrap());
            existing_frag.extend(new_frag);
            self.attach_fragment(existing_frag);
        } else if let Some(key) = self.by_end.remove(new_frag.last().unwrap()) {
            // [existing_frag_start, existing_frag_end] <-- [new_frag_end, new_frag_start]
            let mut existing_frag = self.fragments.remove(&key).unwrap();
            self.by_start.remove(existing_frag.first().unwrap());
            new_frag.reverse();
            existing_frag.extend(new_frag);
            self.attach_fragment(existing_frag);
        } else if let Some(key) = self.by_start.remove(new_frag.first().unwrap()) {
            // [new_frag_end, new_frag_start] --> [existing_frag_start, existing_frag_end]
            let existing_frag = self.fragments.remove(&key).unwrap();
            self.by_end.remove(existing_frag.last().unwrap());
            new_frag.reverse();
            new_frag.extend(existing_frag);
            self.attach_fragment(new_frag);
        } else if let Some(key) = self.by_start.remove(new_frag.last().unwrap()) {
            // [new_frag_start, new_frag_end] --> [existing_frag_start, existing_frag_end]
            let existing_frag = self.fragments.remove(&key).unwrap();
            self.by_end.remove(existing_frag.last().unwrap());
            new_frag.extend(existing_frag);
            self.attach_fragment(new_frag);
        } else {
            // New, detached fragment
            assert!(!new_frag.is_empty());
            let key = self.create_key();
            self.by_start.insert(*new_frag.first().unwrap(), key);
            self.by_end.insert(*new_frag.last().unwrap(), key);
            self.fragments.insert(key, new_frag);
        }
    }

    #[inline]
    pub(super) fn attach(&mut self, line: ILine) {
        let mut frag: Vec<IVec2> = Vec::with_capacity(16);
        frag.push(line.start());
        frag.push(line.end());
        self.attach_fragment(frag)
    }
}

/// A contiguous series of points forming a line segment. The line segment is
/// closed when the first and last point are equal.
#[derive(Clone, Debug, Default)]
pub struct IsoLine {
    pub points: Vec<IVec2>,
}

impl IsoLine {
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.points.len()
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Determine if the [IsoLine] is closed, as indicated by the
    /// first and last points being equal.
    #[inline]
    #[must_use]
    pub fn is_closed(&self) -> bool {
        if self.is_empty() {
            return false;
        }

        let first = self.points.first().unwrap();
        let last = self.points.last().unwrap();

        first == last
    }

    /// Apply Ramer-Douglas-Peucker to produce a simplified subset of point from this [IsoLine].
    #[inline]
    #[must_use]
    pub fn simplify(&self, epsilon: f32) -> IsoLine {
        let points = ramer_douglas_peucker(&self.points, epsilon);
        IsoLine { points }
    }
}

// Adapted from: https://git.sr.ht/~halzy/ramer_douglas_peucker
/*
Copyright 2020 Benjamin G. Halsted <bhalsted@gmail.com>

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */
fn ramer_douglas_peucker(points: &[IVec2], epsilon: f32) -> Vec<IVec2> {
    if points.len() < 3 {
        return Vec::from(points);
    }

    let mut ranges = Vec::<std::ops::RangeInclusive<usize>>::with_capacity(8);

    let mut results = Vec::with_capacity(points.len());
    results.push(points[0]); // We always keep the starting point

    // Set of ranges to work through
    ranges.push(0..=points.len() - 1);

    let mut range_start: usize;
    let mut range_end: usize;
    let mut start: IVec2;
    let mut end: IVec2;
    let mut division_point: usize;
    let mut should_keep_second_half: bool;

    while let Some(range) = ranges.pop() {
        range_start = *range.start();
        range_end = *range.end();

        start = points[range_start];
        end = points[range_end];

        let (max_distance, max_index) = points[range_start + 1..range_end].iter().enumerate().fold(
            (0_f32, 0),
            move |(max_distance, max_index), (index, point)| {
                let distance = {
                    let mut d =
                        distance_to_line(point.as_vec2(), &[start.as_vec2(), end.as_vec2()]);
                    if d.is_zero() {
                        let base = (point.x - start.x) as f32;
                        let height = (point.y - start.y) as f32;
                        d = base.hypot(height);
                    }
                    d
                };

                if distance > max_distance {
                    // new max distance!
                    // +1 to the index because we start enumerate()ing on the 1st element
                    return (distance, index + 1);
                }

                // no new max, pass the previous through
                (max_distance, max_index)
            },
        );

        // If there is a point outside of epsilon, subdivide the range and try again
        if max_distance > epsilon {
            // We add range_start to max_index because the range needs to be in
            // the space of the whole vector and not the range
            division_point = range_start + max_index;

            // Process the second one first to maintain the stack
            // The order of ranges and results are opposite, hence the awkwardness
            should_keep_second_half = division_point - range_start > 2;
            if should_keep_second_half {
                ranges.push(division_point..=range_end);
            }

            if division_point - range_start > 2 {
                ranges.push(range_start..=division_point);
            } else {
                results.push(points[division_point]);
            }

            if !should_keep_second_half {
                results.push(end);
            }
        } else {
            // Keep the end point for the results
            results.push(end);
        }
    }

    results
}
