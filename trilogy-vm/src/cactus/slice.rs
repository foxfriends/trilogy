use super::{Cactus, Pointer};
use std::{marker::PhantomData, ops::Range};

/// A slice of a Cactus stack.
///
/// A slice contains a reference to some shared portion of the Cactus, but does not
/// hold elements of its own. Values cannot be pushed to or popped from a slice, but
/// it is possible to get or set specific indices.
#[derive(Debug)]
pub struct Slice<'a, T> {
    pointer: Pointer<T>,
    _pd: PhantomData<&'a Cactus<T>>,
}

impl<T> Clone for Slice<'_, T> {
    fn clone(&self) -> Self {
        Self {
            pointer: self.pointer.clone(),
            _pd: PhantomData,
        }
    }
}

impl<'a, T> Slice<'a, T> {
    #[inline]
    pub(super) fn new(cactus: &'a Cactus<T>) -> Self {
        let pointer = Pointer::new(cactus);
        Self {
            pointer,
            _pd: PhantomData,
        }
    }

    #[inline]
    pub fn cactus(&self) -> &'a Cactus<T> {
        unsafe { &*self.pointer.cactus }
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
            pointer,
            _pd: PhantomData,
        }
    }

    #[inline]
    pub fn into_pointer(self) -> Pointer<T> {
        self.pointer
    }

    #[inline]
    pub fn pointer(&self) -> &Pointer<T> {
        &self.pointer
    }

    /// Takes a sub-slice of the `Slice`.
    ///
    /// # Panics
    ///
    /// If the range extends beyond the bounds of this `Slice`.
    #[inline]
    pub fn slice(&self, range: Range<usize>) -> Self {
        let pointer = self.pointer.slice(range);
        unsafe { Self::from_pointer(pointer) }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.pointer.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.pointer.is_empty()
    }

    #[inline]
    pub fn truncate(&mut self, len: usize) {
        self.pointer.truncate(len)
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<T>
    where
        T: Clone,
    {
        unsafe { self.pointer.get(index) }
    }

    #[inline]
    pub fn set(&mut self, index: usize, value: T) {
        unsafe {
            self.pointer.set(index, value);
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T>
    where
        T: Clone,
    {
        unsafe { self.pointer.pop() }
    }

    #[inline]
    pub fn peek(&mut self) -> Option<T>
    where
        T: Clone,
    {
        let index = self.len() - 1;
        self.get(index)
    }

    #[inline]
    pub fn pop_n(&mut self, n: usize) -> Option<Vec<T>>
    where
        T: Clone,
    {
        unsafe { self.pointer.pop_n(n) }
    }

    #[inline]
    pub fn append(&mut self, elements: &mut Vec<T>) {
        unsafe { self.pointer.append(elements) }
    }
}
