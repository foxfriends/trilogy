use crate::bytecode::Offset;
use crate::vm::Stack;
use std::fmt::{self, Debug};
use std::hash::Hash;
use std::sync::Arc;

/// A continuation from a Trilogy program.
///
/// From within the program this is seen as an opaque "callable" value.
///
/// It is not possible to construct a value of this type except from within a
/// Trilogy program.
#[derive(Clone)]
pub struct Continuation(Arc<InnerContinuation>);

impl Debug for Continuation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Continuation")
            .field("ip", &self.0.ip)
            .field("stack", &self.0.stack)
            .finish()
    }
}

#[derive(Clone, Debug)]
struct InnerContinuation {
    ip: Offset,
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
    pub(crate) fn new(ip: Offset, stack: Stack) -> Self {
        Self(Arc::new(InnerContinuation { ip, stack }))
    }

    pub(crate) fn ip(&self) -> Offset {
        self.0.ip
    }

    pub(crate) fn stack(&self) -> Stack {
        self.0.stack.clone()
    }
}
