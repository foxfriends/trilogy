mod closure;
mod continuation;
mod native;
mod procedure;
mod threading;

use std::fmt::Display;

pub(crate) use closure::Closure;
pub(crate) use continuation::Continuation;
pub use native::{Native, NativeFunction};
pub(crate) use procedure::Procedure;
pub use threading::Threading;

/// An opaque Trilogy "callable" value.
///
/// This may be a procedure, continuation, closurem, or native function. Such a function may
/// not be called directly from Rust, but can be (or... will someday maybe) processed by a
/// [`VirtualMachine`][crate::VirtualMachine] or [`Execution`][crate::Execution].
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Callable(pub(crate) CallableKind);

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum CallableKind {
    Procedure(Procedure),
    Closure(Closure),
    Continuation(Continuation),
    Native(Native),
}

impl Callable {
    /// Retrieve the native Rust value contained within this callable. From here it may be
    /// downcast into a custom Rust type that has been adapted for use in Trilogy.
    pub fn as_native(&self) -> Option<&Native> {
        match &self.0 {
            CallableKind::Native(native) => Some(native),
            _ => None,
        }
    }
}

impl Display for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            CallableKind::Procedure(value) => write!(f, "{value}"),
            CallableKind::Closure(value) => write!(f, "{value}"),
            CallableKind::Continuation(..) => write!(f, "<anonymous continuation>"),
            CallableKind::Native(..) => write!(f, "<native code>"),
        }
    }
}

impl From<Native> for Callable {
    fn from(value: Native) -> Self {
        Self(CallableKind::Native(value))
    }
}

impl From<Procedure> for Callable {
    fn from(value: Procedure) -> Self {
        Self(CallableKind::Procedure(value))
    }
}

impl From<Continuation> for Callable {
    fn from(value: Continuation) -> Self {
        Self(CallableKind::Continuation(value))
    }
}

impl From<Closure> for Callable {
    fn from(value: Closure) -> Self {
        Self(CallableKind::Closure(value))
    }
}
