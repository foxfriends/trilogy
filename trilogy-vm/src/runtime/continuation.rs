use crate::vm::Stack;
use std::hash::Hash;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Continuation(Arc<InnerContinuation>);

#[derive(Clone, Debug)]
struct InnerContinuation {
    ip: usize,
    stack: Stack,
}

impl Eq for Continuation {}

impl PartialEq for Continuation {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Hash for Continuation {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

impl Continuation {
    pub(crate) fn new(ip: usize, stack: Stack) -> Self {
        Self(Arc::new(InnerContinuation { ip, stack }))
    }

    pub fn ip(&self) -> usize {
        self.0.ip
    }

    pub fn stack(&self) -> Stack {
        self.0.stack.clone()
    }
}
