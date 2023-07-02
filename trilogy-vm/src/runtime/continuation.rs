use crate::vm::Stack;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Continuation(Box<InnerContinuation>);

#[derive(Clone, Debug)]
struct InnerContinuation {
    ip: usize,
    stack: Arc<Mutex<Stack>>,
}

impl Eq for Continuation {}

impl PartialEq for Continuation {
    fn eq(&self, other: &Self) -> bool {
        self.0.ip == other.0.ip && Arc::ptr_eq(&self.0.stack, &other.0.stack)
    }
}

impl Hash for Continuation {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.ip.hash(state);
        Arc::as_ptr(&self.0.stack).hash(state);
    }
}

impl Continuation {
    pub(crate) fn new(ip: usize, stack: Stack) -> Self {
        Self(Box::new(InnerContinuation {
            ip,
            stack: Arc::new(Mutex::new(stack)),
        }))
    }

    pub fn ip(&self) -> usize {
        self.0.ip
    }

    pub fn stack(&self) -> Arc<Mutex<Stack>> {
        self.0.stack.clone()
    }
}
