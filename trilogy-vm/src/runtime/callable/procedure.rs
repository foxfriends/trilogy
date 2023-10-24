use crate::bytecode::Offset;
use crate::vm::stack::Stack;
use std::fmt::{self, Debug, Display};
use std::hash::Hash;
use std::sync::Arc;

/// A procedure, function, or closure from a Trilogy program.
///
/// From within the program this is seen as an opaque "callable" value.
///
/// It is not possible to construct a value of this type except from within a
/// Trilogy program.
#[derive(Clone)]
pub(crate) struct Procedure(Arc<InnerProcedure>);

impl Debug for Procedure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Procedure")
            .field("ip", &self.0.ip)
            .field("stack", &self.0.stack)
            .finish()
    }
}

impl Eq for Procedure {}

impl PartialEq for Procedure {
    fn eq(&self, other: &Self) -> bool {
        if self.0.stack.is_none() && other.0.stack.is_none() {
            self.0.ip == other.0.ip
        } else {
            Arc::ptr_eq(&self.0, &other.0)
        }
    }
}

impl Hash for Procedure {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

#[derive(Clone, Debug)]
struct InnerProcedure {
    ip: Offset,
    stack: Option<Stack>,
}

impl Procedure {
    pub(crate) fn new(pointer: Offset) -> Self {
        Self(Arc::new(InnerProcedure {
            ip: pointer,
            stack: None,
        }))
    }

    pub(crate) fn new_closure(pointer: Offset, stack: Stack) -> Self {
        Self(Arc::new(InnerProcedure {
            ip: pointer,
            stack: Some(stack),
        }))
    }

    pub(crate) fn ip(&self) -> Offset {
        self.0.ip
    }

    pub(crate) fn stack(&self) -> Option<Stack> {
        self.0.stack.clone()
    }
}

impl Display for Procedure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "&({})", self.0.ip)?;
        if self.0.stack.is_some() {
            write!(f, " [closure]")?;
        }
        Ok(())
    }
}
