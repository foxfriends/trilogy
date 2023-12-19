use super::{Cactus, Pointer};
use std::ops::Range;

/// A slice of a Cactus stack.
///
/// A slice contains a reference to some shared portion of the Cactus, but does not
/// hold elements of its own. Values cannot be pushed to or popped from a slice, but
/// it is possible to get or set specific indices.
#[derive(Debug)]
pub struct Slice<'a, T> {
    cactus: &'a Cactus<T>,
    parents: Vec<Range<usize>>,
    len: usize,
}

impl<'a, T> Drop for Slice<'a, T> {
    fn drop(&mut self) {
        self.cactus.release_ranges(&self.parents);
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

impl<'a, T> Slice<'a, T> {
    #[inline]
    pub(super) fn new(cactus: &'a Cactus<T>) -> Self {
        Self {
            cactus,
            parents: vec![],
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
    pub unsafe fn from_pointer(cactus: &'a Cactus<T>, pointer: Pointer<T>) -> Self {
        Slice {
            cactus,
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
    pub unsafe fn reacquire(&self) {
        self.cactus.acquire_ranges(&self.parents);
    }

    #[inline]
    pub fn into_pointer(self) -> Pointer<T> {
        Pointer::new(self.parents.clone(), self.len)
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
        let parent = self.parents.last_mut()?;
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
                popped += parent.len();
                ranges.push(parent);
            }
        }
        ranges.reverse();
        self.len -= n;
        unsafe { self.cactus.get_release_ranges(&ranges) }
    }

    #[inline]
    pub fn append(&mut self, elements: &mut Vec<T>) {
        self.len += elements.len();
        let range = self.cactus.append(elements);
        self.parents.push(range);
    }
}
