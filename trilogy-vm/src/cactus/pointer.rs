use super::{Cactus, RangeMap};
use std::fmt::Debug;
use std::ops::Range;

/// A "raw pointer" to part of a Cactus stack.
///
/// This pointer points to part of the shared Cactus stack without actually containing
/// any reference to it. It is up to the user of this value to ensure that the intended
/// Cactus does not drop any of the values this pointer points to.
pub struct Pointer<T> {
    pub(super) cactus: *const Cactus<T>,
    pub(super) parents: RangeMap<bool>,
    pub(super) len: usize,
}

impl<T> Debug for Pointer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pointer")
            .field("cactus", &self.cactus)
            .field("parents", &self.parents)
            .field("len", &self.len)
            .finish()
    }
}

impl<T> Clone for Pointer<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            cactus: self.cactus,
            parents: self.parents.clone(),
            len: self.len,
        }
    }
}

impl<T> Pointer<T> {
    #[inline]
    pub(super) fn new(cactus: &Cactus<T>) -> Self {
        Self {
            cactus,
            parents: Default::default(),
            len: 0,
        }
    }

    /// Gets the ranges that this pointer points to.
    #[inline]
    pub fn ranges(&self) -> &RangeMap<bool> {
        &self.parents
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn truncate(&mut self, len: usize) {
        while self.len > len {
            let (parent, _) = self.parents.last_range().unwrap();
            if parent.len() <= self.len - len {
                self.len -= parent.len();
                self.parents.remove(parent.clone());
            } else {
                let to_pop = self.len - len;
                self.parents.remove(parent.end - to_pop..parent.end);
                self.len -= to_pop;
            }
        }
    }

    unsafe fn cactus_ref(&self) -> &Cactus<T> {
        unsafe { &*self.cactus }
    }

    #[inline]
    pub unsafe fn get(&self, index: usize) -> Option<T>
    where
        T: Clone,
    {
        let parent_index = self.resolve_index(index)?;
        self.cactus_ref().get(parent_index)
    }

    #[inline]
    pub unsafe fn set(&mut self, index: usize, value: T) {
        let parent_index = self.resolve_index(index).unwrap();
        self.cactus_ref().set(parent_index, value);
    }

    #[inline]
    pub unsafe fn pop(&mut self) -> Option<T>
    where
        T: Clone,
    {
        let index = self.parents.len() - 1;
        let value = self.cactus_ref().get(index)?;
        self.parents.pop();
        self.len -= 1;
        Some(value)
    }

    #[inline]
    fn resolve_index(&self, mut index: usize) -> Option<usize> {
        for (range, _) in self.parents.iter().filter(|(_, v)| *v) {
            if range.len() > index {
                return Some(range.start + index);
            }
            index -= range.len();
        }
        None
    }

    /// Takes a sub-slice of the `Pointer`.
    ///
    /// # Panics
    ///
    /// If the range extends beyond the bounds of this `Slice`.
    #[inline]
    pub fn slice(&self, mut range: Range<usize>) -> Self {
        let len = range.len();
        let mut i = 0;
        let mut sliced_parents = RangeMap::default();
        for (parent, _) in self.parents.iter().filter(|(_, v)| *v) {
            if i + parent.len() > range.start {
                let overlap_start = parent.start + range.start - i;
                let overlapping_range =
                    overlap_start..usize::min(parent.end, overlap_start + range.len());
                range.start += overlapping_range.len();
                sliced_parents.insert(overlapping_range, true);
            }
            i += parent.len();
            if i >= range.end {
                break;
            }
        }
        Self {
            cactus: self.cactus,
            parents: sliced_parents,
            len,
        }
    }

    #[inline]
    pub unsafe fn pop_n(&mut self, n: usize) -> Option<Vec<T>>
    where
        T: Clone,
    {
        if self.len < n {
            return None;
        }
        let mut ranges = vec![];
        let mut popped = 0;
        while popped < n {
            let (parent, _) = self.parents.last_range().unwrap();
            if popped + parent.len() > n {
                let from_range = n - popped;
                self.parents.remove(parent.end - from_range..parent.end);
                ranges.push(parent.end - from_range..parent.end);
                break;
            } else {
                popped += parent.len();
                self.parents.remove(parent.clone());
                ranges.push(parent);
            }
        }
        ranges.reverse();
        self.len -= n;
        self.cactus_ref().get_ranges(ranges)
    }

    #[inline]
    pub unsafe fn append(&mut self, elements: &mut Vec<T>) {
        if elements.is_empty() {
            return;
        }
        self.len += elements.len();
        let range = self.cactus_ref().append(elements);
        self.parents.insert(range, true);
    }
}
