use super::super::RefCount;
use crate::bytecode::Offset;
use crate::vm::stack::Stack;
use std::fmt::{self, Debug, Display};
use std::hash::Hash;

/// A closure from a Trilogy program.
///
/// From within the program this is seen as an opaque "callable" value.
///
/// It is not possible to construct a value of this type except from within a
/// Trilogy program.
#[derive(Clone)]
pub(crate) struct Closure(RefCount<InnerClosure>);

impl Debug for Closure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Closure")
            .field("ip", &self.0.ip)
            .field("stack", &self.0.stack)
            .finish()
    }
}

#[derive(Clone, Debug)]
struct InnerClosure {
    ip: Offset,
    stack: Stack,
}

impl InnerClosure {
    fn new(ip: Offset, stack: Stack) -> Self {
        #[cfg(feature = "stats")]
        crate::GLOBAL_STATS
            .closures_allocated
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self { ip, stack }
    }
}

#[cfg(feature = "stats")]
impl Drop for InnerClosure {
    fn drop(&mut self) {
        crate::GLOBAL_STATS
            .closures_freed
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Eq for Closure {}

impl PartialEq for Closure {
    fn eq(&self, other: &Self) -> bool {
        RefCount::ptr_eq(&self.0, &other.0)
    }
}

impl Hash for Closure {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        RefCount::as_ptr(&self.0).hash(state);
    }
}

impl Closure {
    pub(crate) fn new(ip: Offset, stack: Stack) -> Self {
        Self(RefCount::new(InnerClosure::new(ip, stack)))
    }

    pub(crate) fn ip(&self) -> Offset {
        self.0.ip
    }

    pub(crate) fn stack(&self) -> &Stack {
        &self.0.stack
    }
}

impl Display for Closure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "&({}) [closure]", self.0.ip)
    }
}
