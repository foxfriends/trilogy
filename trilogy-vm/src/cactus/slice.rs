use super::Cactus;
use std::ops::Range;

/// A slice of a Cactus stack.
///
/// A slice contains a reference to some shared portion of the Cactus, but does not
/// hold elements of its own. Values cannot be pushed to or popped from a slice, but
/// it is possible to get or set specific indices.
pub struct Slice<'a, T> {
    cactus: &'a Cactus<T>,
    parents: Vec<Range<usize>>,
    len: usize,
}

impl<'a, T> Drop for Slice<'a, T> {
    fn drop(&mut self) {
        for range in self.parents {
            self.cactus.release_range(range);
        }
    }
}

impl<T> Clone for Slice<'_, T> {
    fn clone(&self) -> Self {
        self.cactus.acquire_ranges(&self.parents);
        Self {
            cactus: self.cactus,
            parents: self.parents.clone(),
            len: self.len,
        }
    }
}

impl<T> Slice<'_, T> {
    pub(super) fn new(cactus: &Cactus<T>) -> Self {
        Self {
            cactus,
            parents: vec![],
            len: 0,
        }
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
    pub(super) fn resolve_index(&self, mut index: usize) -> Option<usize> {
        for range in &self.parents {
            if range.len() > index {
                return Some(range.start + index);
            }
            index -= range.len();
        }
        None
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<T>
    where
        T: Clone,
    {
        let parent_index = self.resolve_index(index)?;
        unsafe { self.cactus.get_unchecked(parent_index) }
    }

    #[inline]
    pub fn set(&mut self, index: usize, value: T) {
        let parent_index = self.resolve_index(index).unwrap();
        unsafe {
            self.cactus.set_unchecked(parent_index, value);
        }
    }

    pub fn pop(&mut self) -> Option<T>
    where
        T: Clone,
    {
        let mut parent = self.parents.last_mut()?;
        let index = parent.end;
        let value = unsafe { self.cactus.get_release(index) };
        parent.end -= 1;
        if parent.end == parent.start {
            self.parents.pop();
        }
        self.len -= 1;
        Some(value)
    }

    pub fn pop_n(&mut self, n: usize) -> Vec<T>
    where
        T: Clone,
    {
        let mut ranges = vec![];
        let mut popped = 0;
        while popped < n {
            let parent = self
                .parents
                .pop()
                .expect("attempted to pop elements out of range");
            if popped + parent.len() > n {
                let from_range = n - popped;
                self.parents.push(parent.start..parent.start + from_range);
                ranges.push(parent.start + from_range..parent.end);
                break;
            } else {
                ranges.push(parent);
            }
        }
        ranges.reverse();
        unsafe { self.cactus.get_release_ranges(&ranges) }
    }

    #[inline]
    pub fn append(&mut self, elements: &mut Vec<T>) {
        let range = self.cactus.append(elements);
        self.parents.push(range);
    }
}
