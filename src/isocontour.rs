use crate::ILine;
use bevy_math::IVec2;
use std::collections::HashMap;

pub type FragmentKey = usize;

pub type Fragment = Vec<IVec2>;

pub(super) struct FragmentAccumulator {
    next_key: FragmentKey,
    fragments: HashMap<FragmentKey, Fragment>,
    by_start: HashMap<IVec2, FragmentKey>,
    by_end: HashMap<IVec2, FragmentKey>,
}

impl FragmentAccumulator {
    pub(super) fn new(size: usize) -> Self {
        Self {
            next_key: 0,
            fragments: HashMap::with_capacity(size),
            by_start: HashMap::with_capacity(size),
            by_end: HashMap::with_capacity(size),
        }
    }

    pub(super) fn result(self) -> Vec<IsoLine> {
        assert_eq!(self.by_start.len(), self.fragments.len());
        assert_eq!(self.by_end.len(), self.fragments.len());
        self.fragments
            .into_values()
            .map(|frag| IsoLine { points: frag })
            .collect()
    }

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

    pub(super) fn attach(&mut self, line: ILine) {
        let mut frag: Vec<IVec2> = Vec::with_capacity(16);
        frag.push(line.start());
        frag.push(line.end());
        self.attach_fragment(frag)
    }
}

#[derive(Clone, Debug, Default)]
pub struct IsoLine {
    pub points: Vec<IVec2>,
}

impl IsoLine {
    #[inline]
    pub fn len(&self) -> usize {
        self.points.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    #[inline]
    pub fn is_closed(&self) -> bool {
        if self.is_empty() {
            return false;
        }

        let first = self.points.first().unwrap();
        let last = self.points.last().unwrap();

        first == last
    }

    pub fn simplify(&self) -> IsoLine {
        // Apply douglas peucker
        // TODO

        self.clone()
    }
}
