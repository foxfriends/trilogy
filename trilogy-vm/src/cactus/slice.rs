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

impl<T: Clone> Clone for Slice<'_, T> {
    fn clone(&self) -> Self {
        let ranges = self.cactus.ranges.lock().unwrap();
        for range in self.parents {
            self.cactus.acquire_range(range);
        }
        Self {
            cactus: self.cactus,
            parents: self.parents.clone(),
            len: self.len,
        }
    }
}

impl<T> Slice<'_, T> {
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    fn resolve_index(&self, mut index: usize) -> Option<usize> {
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
}
