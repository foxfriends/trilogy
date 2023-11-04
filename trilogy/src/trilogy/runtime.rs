use trilogy_codegen::YIELD;
use trilogy_vm::{Atom, ErrorKind, Execution, Native, NativeFunction, Value};

/// A handle to the Trilogy language runtime, allowing native functions written
/// in Rust to interact effectively with the running Trilogy program.
///
/// Due to Trilogy's control flow being more flexible than Rust's, we
/// cannot effectively use Rust control flow to manipulate a Trilogy
/// program. Instead, control flow is done in continuation passing style
/// using specific continuations provided by the runtime.
pub struct Runtime<'prog, 'ex>(&'ex mut trilogy_vm::Execution<'prog>);

impl<'prog, 'ex> Runtime<'prog, 'ex> {
    #[doc(hidden)]
    pub fn new(inner: &'ex mut trilogy_vm::Execution<'prog>) -> Self {
        Self(inner)
    }
}

impl<'prog, 'ex> Runtime<'prog, 'ex> {
    pub fn atom(&self, tag: &str) -> Atom {
        self.0.atom(tag)
    }

    pub fn atom_anon(&self, tag: &str) -> Atom {
        self.0.atom_anon(tag)
    }

    /// The equivalent of the yield operator, allowing a native function to
    /// yield an effect.
    pub fn r#yield<F>(self, value: impl Into<Value>, mut f: F) -> crate::Result<()>
    where
        F: FnMut(Runtime, Value) -> crate::Result<()> + Sync + Send + 'static,
    {
        let y = self.0.procedure(YIELD)?;
        let effect = value.into();
        self.0
            .callback(y, vec![effect], move |ex, val| f(Runtime::new(ex), val))
    }

    /// The equivalent of the return operator, allowing a native function to return a value.
    ///
    /// Calling return more than once is not permitted.
    pub fn r#return(self, value: impl Into<Value>) -> crate::Result<()> {
        self.0.r#return(value.into())
    }

    /// Construct a Trilogy closure from a Rust closure.
    pub fn closure<F, const N: usize>(self, cb: F) -> Value
    where
        F: FnMut(Runtime, [Value; N]) -> crate::Result<()> + Sync + Send + 'static,
    {
        struct Callback<F, const N: usize>(F);

        impl<F, const N: usize> NativeFunction for Callback<F, N>
        where
            F: FnMut(Runtime, [Value; N]) -> crate::Result<()> + Sync + Send + 'static,
        {
            fn name() -> &'static str
            where
                Self: Sized,
            {
                "<anonymous>"
            }

            fn arity(&self) -> usize {
                N
            }

            fn call(&mut self, ex: &mut Execution, values: Vec<Value>) -> crate::Result<()> {
                if values.len() == N {
                    let args = values.try_into().unwrap();
                    self.0(Runtime::new(ex), args)
                } else {
                    Err(ex.error(ErrorKind::RuntimeError(
                        "a native closure was called with the incorrect arity".to_owned(),
                    )))
                }
            }
        }

        Value::from(Native::from(Callback(cb)))
    }
}
