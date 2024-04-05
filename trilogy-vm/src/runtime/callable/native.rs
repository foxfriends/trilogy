use super::super::RefCount;
use crate::{Error, Execution, ReferentialEq, StructuralEq, Value};
use std::any::{Any, TypeId};
use std::fmt::{self, Debug};
use std::hash::{self, Hash};
use std::sync::Mutex;

#[cfg(not(feature = "multithread"))]
pub trait Threading {}
#[cfg(not(feature = "multithread"))]
impl<T> Threading for T {}

#[cfg(feature = "multithread")]
pub trait Threading: Send + Sync {}
#[cfg(feature = "multithread")]
impl<T: Send + Sync> Threading for T {}

/// Trait allowing Rust functions to be called by Trilogy programs.
///
/// Implementing this trait manually is not recommended, see instead the macro
/// `#[proc]` attribute macro from the `trilogy` crate.
pub trait NativeFunction: Threading + Any {
    #[doc(hidden)]
    fn call(&mut self, ex: &mut Execution, input: Vec<Value>) -> Result<(), Error>;

    #[doc(hidden)]
    fn arity(&self) -> usize;
}

/// A native (Rust) function, which has been bridged to be callable from Trilogy.
///
/// From within the program this is seen as an opaque "callable" value.
#[derive(Clone)]
pub struct Native(RefCount<Mutex<dyn NativeFunction + 'static>>);

impl Debug for Native {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "<native code>".fmt(f)
    }
}

impl Eq for Native {}

impl PartialEq for Native {
    fn eq(&self, other: &Self) -> bool {
        RefCount::ptr_eq(&self.0, &other.0)
    }
}

impl Hash for Native {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        RefCount::as_ptr(&self.0).hash(state);
    }
}

impl ReferentialEq for Native {
    fn eq(&self, other: &Self) -> bool {
        RefCount::ptr_eq(&self.0, &other.0)
    }
}

impl StructuralEq for Native {
    fn eq(&self, other: &Self) -> bool {
        RefCount::ptr_eq(&self.0, &other.0)
    }
}

impl Native {
    pub(crate) fn call(&self, ex: &mut Execution, args: Vec<Value>) -> Result<(), Error> {
        let mut native = self.0.lock().unwrap();
        native.call(ex, args)
    }

    /// Attempts to downcast this native value to its wrapped Rust type.
    pub fn downcast<T>(&self) -> Option<RefCount<Mutex<T>>>
    where
        T: Any,
    {
        // See:
        //     https://stackoverflow.com/questions/68173030/how-to-do-runtime-polymorphism-using-arcmutexdyn-sometrait-in-rust
        if (*self.0.lock().unwrap()).type_id() == TypeId::of::<T>() {
            let raw: *const Mutex<dyn NativeFunction> = RefCount::into_raw(self.0.clone());
            let raw: *const Mutex<T> = raw.cast();

            // SAFETY: This is safe because the pointer orignally came from a Rc/Arc
            // with the same size and alignment since we've checked (via Any) that
            // the object within is the type being casted to.
            Some(unsafe { RefCount::from_raw(raw) })
        } else {
            None
        }
    }
}

impl<T: NativeFunction + 'static> From<T> for Native {
    fn from(value: T) -> Self {
        Self(RefCount::new(Mutex::new(value)))
    }
}
