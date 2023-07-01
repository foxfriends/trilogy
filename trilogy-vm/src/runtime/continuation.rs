use crate::vm::Stack;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Continuation {
    ip: usize,
    stack: Arc<Mutex<Stack>>,
}

impl Eq for Continuation {}

impl PartialEq for Continuation {
    fn eq(&self, other: &Self) -> bool {
        self.ip == other.ip && Arc::ptr_eq(&self.stack, &other.stack)
    }
}

impl Hash for Continuation {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ip.hash(state);
        Arc::as_ptr(&self.stack).hash(state);
    }
}

impl Continuation {
    pub(crate) fn new(ip: usize, stack: Stack) -> Self {
        Self {
            ip,
            stack: Arc::new(Mutex::new(stack)),
        }
    }

    pub fn ip(&self) -> usize {
        self.ip
    }

    pub fn stack(&self) -> Arc<Mutex<Stack>> {
        self.stack.clone()
    }
}
