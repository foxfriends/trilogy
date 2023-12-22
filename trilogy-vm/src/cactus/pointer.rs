use super::{Cactus, RangeMap};
use std::fmt::Debug;
use std::marker::PhantomData;

/// A "raw pointer" to part of a Cactus stack.
///
/// This pointer points to part of the shared Cactus stack without actually containing
/// any reference to it. It is up to the user of this value to ensure that the intended
/// Cactus does not drop any of the values this pointer points to.
pub struct Pointer<T> {
    pub(super) cactus: *const Cactus<T>,
    pub(super) parents: RangeMap<bool>,
    pub(super) len: usize,
    _pd: PhantomData<T>,
}

impl<T> Debug for Pointer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pointer")
            .field("parents", &self.parents)
            .field("len", &self.len)
            .finish_non_exhaustive()
    }
}

impl<T> Clone for Pointer<T> {
    fn clone(&self) -> Self {
        Self {
            cactus: self.cactus,
            parents: self.parents.clone(),
            len: self.len,
            _pd: PhantomData,
        }
    }
}

impl<T> Pointer<T> {
    pub(super) fn new(cactus: &Cactus<T>, parents: RangeMap<bool>, len: usize) -> Self {
        Self {
            cactus,
            parents,
            len,
            _pd: PhantomData,
        }
    }
}
