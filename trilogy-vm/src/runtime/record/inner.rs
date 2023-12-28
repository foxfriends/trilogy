use crate::Value;
use std::collections::HashMap;

#[derive(Debug)]
pub(super) struct RecordInner(HashMap<Value, Value>);

impl std::ops::Deref for RecordInner {
    type Target = HashMap<Value, Value>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for RecordInner {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for RecordInner {
    #[inline(always)]
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl RecordInner {
    #[inline(always)]
    pub(super) fn new(value: HashMap<Value, Value>) -> Self {
        #[cfg(feature = "stats")]
        crate::GLOBAL_STATS
            .records_allocated
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self(value)
    }
}

#[cfg(feature = "stats")]
impl Drop for RecordInner {
    fn drop(&mut self) {
        crate::GLOBAL_STATS
            .records_freed
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}
