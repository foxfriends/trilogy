use crate::cactus::Cactus;
use crate::vm::stack::{Stack, StackCell};
use bitvec::prelude::*;

#[derive(Clone)]
pub(crate) struct GarbageCollector<'a> {
    cactus: &'a Cactus<StackCell>,
}

impl<'a> GarbageCollector<'a> {
    pub fn new(cactus: &'a Cactus<StackCell>) -> Self {
        Self { cactus }
    }

    pub fn collect_garbage(&self, stacks: &[&Stack<'a>]) {
        let mut marks = bitvec![0; self.cactus.len()];
        for stack in stacks {
            for frame in stack.frames() {
                if let Some(slice) = &frame.slice {
                    slice.pointer();
                }
            }
            let branch = stack.active();
        }
    }
}
