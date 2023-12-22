use super::{Cactus, Pointer, RangeMap};
use std::ops::Range;

/// A slice of a Cactus stack.
///
/// A slice contains a reference to some shared portion of the Cactus, but does not
/// hold elements of its own. Values cannot be pushed to or popped from a slice, but
/// it is possible to get or set specific indices.
#[derive(Debug)]
pub struct Slice<'a, T> {
    cactus: &'a Cactus<T>,
    parents: RangeMap<bool>,
    len: usize,
}

impl<'a, T> Drop for Slice<'a, T> {
    fn drop(&mut self) {
        if !self.parents.is_empty() {
            self.release();
        }
    }
}

impl<T> Clone for Slice<'_, T> {
    fn clone(&self) -> Self {
        self.reacquire();
        Self {
            cactus: self.cactus,
            parents: self.parents.clone(),
            len: self.len,
        }
    }
}

impl<'a, T> Slice<'a, T> {
    #[inline]
    pub(super) fn new(cactus: &'a Cactus<T>) -> Self {
        Self {
            cactus,
            parents: RangeMap::default(),
            len: 0,
        }
    }

    #[inline]
    pub fn cactus(&self) -> &'a Cactus<T> {
        self.cactus
    }

    /// Constructs a slice from a pointer and the cactus it is pointing to.
    ///
    /// # Safety
    ///
    /// The pointer should have been created using [`into_pointer`][Self::into_pointer]
    /// from a Slice on the same Cactus that has been provided.
    ///
    /// The caller must also ensure that the Cactus has not yet freed the elements that
    /// this pointer points to.
    ///
    /// The invariants required here are very similar to those of [`Arc::from_raw`][std::sync::Arc::from_raw]
    #[inline]
    pub unsafe fn from_pointer(pointer: Pointer<T>) -> Self {
        Slice {
            cactus: unsafe { &*pointer.cactus },
            parents: pointer.parents,
            len: pointer.len,
        }
    }

    /// Increases the reference counts for all values pointed to by this slice.
    ///
    /// # Safety
    ///
    /// The caller must ensure that this does not reacquire elements already freed.
    /// It is also recommended to ensure that the re-acquired elements are correctly
    /// released.
    #[inline]
    pub fn reacquire(&self) {
        self.cactus.acquire_ranges(
            &self
                .parents
                .iter()
                .filter(|(_, v)| *v)
                .map(|(r, _)| r)
                .collect::<Vec<_>>(),
        );
    }

    /// Increases the reference counts for all values pointed to by this slice.
    ///
    /// # Safety
    ///
    /// The caller must ensure that this does not reacquire elements already freed.
    /// It is also recommended to ensure that the re-acquired elements are correctly
    /// released.
    #[inline]
    pub fn release(&self) {
        self.cactus.release_ranges(
            &self
                .parents
                .iter()
                .filter(|(_, v)| *v)
                .map(|(r, _)| r)
                .collect::<Vec<_>>(),
        );
    }

    #[inline]
    pub fn into_pointer(mut self) -> Pointer<T> {
        let parents = std::mem::take(&mut self.parents);
        Pointer::new(self.cactus, parents, self.len)
    }

    /// Takes a sub-slice of the `Slice`.
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
        let new = Self {
            cactus: self.cactus,
            parents: sliced_parents,
            len,
        };
        new.reacquire();
        new
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
        let mut to_release = vec![];
        while self.len > len {
            let (parent, _) = self.parents.last_range().unwrap();
            if parent.len() <= self.len - len {
                self.len -= parent.len();
                self.parents.remove(parent.clone());
                to_release.push(parent);
            } else {
                let to_pop = self.len - len;
                to_release.push(parent.end - to_pop..parent.end);
                self.parents.remove(parent.end - to_pop..parent.end);
                self.len -= to_pop;
            }
        }
        self.cactus.release_ranges(&to_release);
    }

    #[inline]
    pub(super) fn resolve_index(&self, mut index: usize) -> Option<usize> {
        for (range, _) in self.parents.iter().filter(|(_, v)| *v) {
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
        let index = self.parents.len() - 1;
        let value = unsafe { self.cactus.get_release(index) };
        self.parents.remove(index..index + 1);
        self.len -= 1;
        Some(value)
    }

    pub fn peek(&mut self) -> Option<T>
    where
        T: Clone,
    {
        let index = self.parents.len() - 1;
        unsafe { self.cactus.get_unchecked(index) }
    }

    pub fn pop_n(&mut self, n: usize) -> Result<Vec<T>, Vec<T>>
    where
        T: Clone,
    {
        let mut ranges = vec![];
        let mut popped = 0;
        while popped < n {
            let parent = match self.parents.last_range() {
                Some((parent, _)) => parent,
                None => {
                    self.len = 0;
                    return Err(unsafe { self.cactus.get_release_ranges(&ranges) });
                }
            };
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
        Ok(unsafe { self.cactus.get_release_ranges(&ranges) })
    }

    #[inline]
    pub fn append(&mut self, elements: &mut Vec<T>) {
        if elements.is_empty() {
            return;
        }
        self.len += elements.len();
        let range = self.cactus.append(elements);
        self.parents.insert(range, true);
    }
}
