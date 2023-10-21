mod continuation;
mod native;
mod procedure;

use std::fmt::Display;

pub(crate) use continuation::Continuation;
pub use native::{Native, NativeFunction};
pub(crate) use procedure::Procedure;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Callable(pub(crate) CallableKind);

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum CallableKind {
    Procedure(Procedure),
    Continuation(Continuation),
    Native(Native),
}

impl Display for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            CallableKind::Procedure(value) => write!(f, "{value}"),
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
