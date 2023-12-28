use super::super::RefCount;
use crate::bytecode::Offset;
use crate::cactus::{Pointer, Slice};
use crate::vm::stack::StackCell;
use std::fmt::{self, Debug, Display};
use std::hash::Hash;

/// A closure from a Trilogy program.
///
/// From within the program this is seen as an opaque "callable" value.
///
/// It is not possible to construct a value of this type except from within a
/// Trilogy program. If held beyond the end of the execution of the Trilogy
/// program, it is no longer valid to be called.
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
    #[inline]
    pub(crate) fn new(ip: Offset, stack: Slice<'_, StackCell>) -> Self {
        Self(RefCount::new(InnerClosure::new(ip, stack)))
    }

    /// Returns the ID of the underlying closure instance. This ID will remain
    /// stable for the lifetime of each array instance, and is unique per
    /// instance.
    #[inline]
    pub fn id(&self) -> usize {
        RefCount::as_ptr(&self.0) as usize
    }

    #[inline]
    pub fn stack_pointer(&self) -> &Pointer<StackCell> {
        &self.0.stack
    }

    #[inline]
    pub(crate) fn ip(&self) -> Offset {
        self.0.ip
    }

    #[inline]
    pub(crate) unsafe fn stack<'a>(&self) -> Slice<'a, StackCell> {
        Slice::from_pointer(self.0.stack.clone())
    }
}

impl Display for Closure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "&({}) [closure]", self.0.ip)
    }
}

#[derive(Clone)]
struct InnerClosure {
    ip: Offset,
    stack: Pointer<StackCell>,
}

impl InnerClosure {
    #[inline(always)]
    fn new(ip: Offset, stack: Slice<'_, StackCell>) -> Self {
        #[cfg(feature = "stats")]
        crate::GLOBAL_STATS
            .closures_allocated
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self {
            ip,
            stack: stack.into_pointer(),
        }
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
