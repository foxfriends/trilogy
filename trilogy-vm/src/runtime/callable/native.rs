use crate::{Execution, ReferentialEq, StructuralEq, Value};
use std::fmt::{self, Debug};
use std::hash::{self, Hash};
use std::sync::Arc;

/// Trait allowing Rust functions to be called by Trilogy programs.
///
/// Implementing this trait manually is not recommended, see instead the macro
/// `#[proc]` attribute macro from the `trilogy` crate.
pub trait NativeFunction: Send + Sync {
    #[doc(hidden)]
    fn name() -> &'static str
    where
        Self: Sized;

    #[doc(hidden)]
    fn call(&self, ex: Execution, input: Vec<Value>) -> Value;

    #[doc(hidden)]
    fn arity(&self) -> usize;
}

/// A native (Rust) function, which has been bridged to be callable from Trilogy.
///
/// From within the program this is seen as an opaque "callable" value.
#[derive(Clone)]
pub struct Native(Arc<dyn NativeFunction + 'static>);

impl Debug for Native {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "<native code>".fmt(f)
    }
}

impl Eq for Native {}

impl PartialEq for Native {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Hash for Native {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

impl ReferentialEq for Native {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl StructuralEq for Native {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Native {
    pub(crate) fn call(&self, ex: Execution, args: Vec<Value>) -> Value {
        assert_eq!(args.len(), self.0.arity());
        self.0.call(ex, args)
    }
}

impl<T: NativeFunction + 'static> From<T> for Native {
    fn from(value: T) -> Self {
        Self(Arc::new(value))
    }
}
