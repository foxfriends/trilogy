use std::marker::PhantomData;
use std::ops::Range;

/// A "raw pointer" to part of a Cactus stack.
///
/// This pointer points to part of the shared Cactus stack without actually containing
/// any reference to it. It is up to the user of this value to ensure that the intended
/// Cactus does not drop any of the values this pointer points to.
#[derive(Debug)]
pub struct Pointer<T> {
    pub(super) parents: Vec<Range<usize>>,
    pub(super) len: usize,
    _pd: PhantomData<T>,
}

impl<T> Clone for Pointer<T> {
    fn clone(&self) -> Self {
        Self {
            parents: self.parents.clone(),
            len: self.len,
            _pd: PhantomData,
        }
    }
}

impl<T> Pointer<T> {
    pub(super) fn new(parents: Vec<Range<usize>>, len: usize) -> Self {
        Self {
            parents,
            len,
            _pd: PhantomData,
        }
    }
}
