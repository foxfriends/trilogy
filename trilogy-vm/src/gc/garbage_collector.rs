use crate::cactus::Cactus;
use crate::vm::stack::StackCell;

#[derive(Clone)]
pub(crate) struct GarbageCollector<'a> {
    cactus: &'a Cactus<StackCell>,
}

impl<'a> GarbageCollector<'a> {
    pub fn new(cactus: &'a Cactus<StackCell>) -> Self {
        Self { cactus }
    }

    pub fn collect_garbage(&self) {}
}
