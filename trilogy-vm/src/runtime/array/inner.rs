use crate::Value;
use std::fmt::{self, Debug};

pub(super) struct ArrayInner(Vec<Value>);

impl Debug for ArrayInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(&self.0).finish()
    }
}

impl Default for ArrayInner {
    #[inline]
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl std::ops::Deref for ArrayInner {
    type Target = Vec<Value>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ArrayInner {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ArrayInner {
    #[inline]
    pub(super) fn new(array: Vec<Value>) -> Self {
        #[cfg(feature = "stats")]
        crate::GLOBAL_STATS
            .arrays_allocated
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self(array)
    }
}

#[cfg(feature = "stats")]
impl Drop for ArrayInner {
    fn drop(&mut self) {
        crate::GLOBAL_STATS
            .arrays_freed
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}
