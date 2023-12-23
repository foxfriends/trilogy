use super::Dumpster;
use crate::cactus::Cactus;
use crate::vm::stack::StackCell;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct GarbageCollector<'a> {
    cactus: &'a Cactus<StackCell>,
    dumpster: Arc<Dumpster>,
}

impl<'a> GarbageCollector<'a> {
    pub fn new(cactus: &'a Cactus<StackCell>) -> Self {
        Self {
            cactus,
            dumpster: Default::default(),
        }
    }

    pub fn collect_garbage(&self) {
        // First, collect garbage that we knew about
        let trash = std::mem::take(&mut *self.dumpster.trash_mut());
        self.cactus.release_all(trash);
    }

    pub fn dumpster(&self) -> Arc<Dumpster> {
        self.dumpster.clone()
    }
}
