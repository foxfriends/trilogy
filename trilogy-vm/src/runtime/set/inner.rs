use crate::Value;
use std::collections::HashSet;

#[derive(Debug)]
pub(super) struct SetInner(HashSet<Value>);

impl std::ops::Deref for SetInner {
    type Target = HashSet<Value>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for SetInner {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for SetInner {
    #[inline]
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl SetInner {
    #[inline]
    pub(super) fn new(value: HashSet<Value>) -> Self {
        #[cfg(feature = "stats")]
        crate::GLOBAL_STATS
            .sets_allocated
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self(value)
    }
}

#[cfg(feature = "stats")]
impl Drop for SetInner {
    fn drop(&mut self) {
        crate::GLOBAL_STATS
            .sets_freed
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}
