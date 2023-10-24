use crate::bytecode::Offset;
use crate::vm::stack::Stack;
use std::fmt::{self, Debug, Display};
use std::hash::Hash;
use std::sync::Arc;

/// A closure from a Trilogy program.
///
/// From within the program this is seen as an opaque "callable" value.
///
/// It is not possible to construct a value of this type except from within a
/// Trilogy program.
#[derive(Clone)]
pub(crate) struct Closure {
    ip: Offset,
    stack: Arc<Stack>,
}

impl Debug for Closure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Closure")
            .field("ip", &self.ip)
            .field("stack", &self.stack)
            .finish()
    }
}

impl Eq for Closure {}

impl PartialEq for Closure {
    fn eq(&self, other: &Self) -> bool {
        self.ip == other.ip && Arc::ptr_eq(&self.stack, &other.stack)
    }
}

impl Hash for Closure {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ip.hash(state);
        Arc::as_ptr(&self.stack).hash(state);
    }
}

impl Closure {
    pub(crate) fn new(pointer: Offset, stack: Stack) -> Self {
        Self {
            ip: pointer,
            stack: Arc::new(stack),
        }
    }

    pub(crate) fn ip(&self) -> Offset {
        self.ip
    }

    pub(crate) fn stack(&self) -> &Stack {
        &self.stack
    }
}

impl Display for Closure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "&({}) [closure]", self.ip)
    }
}
