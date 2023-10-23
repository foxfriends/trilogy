use std::marker::PhantomData;
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
    pub fn r#yield<F>(&mut self, value: impl Into<Value>, mut f: F) -> crate::Result<()>
    where
        F: FnMut(&mut Runtime, Value) -> crate::Result<()> + Sync + Send + 'static,
    {
        let effect = value.into();
        // TODO: obviously the function shouldn't be the effect, but I just need
        // a value for now so this compiles, and I don't want to use todo!()
        self.0
            .callback(effect.clone(), vec![effect], move |ex, val| {
                f(&mut Runtime::new(ex), val)
            })
    }

    /// The equivalent of the return operator, allowing a native function to
    /// return a value. As in Trilogy, this can possibly  happen more than once.
    pub fn r#return(&mut self, value: impl Into<Value>) -> crate::Result<()> {
        let value = value.into();
        // TODO: obviously the function shouldn't be the effect, but I just need
        // a value for now so this compiles, and I don't want to use todo!()
        self.0.callback(value.clone(), vec![value], |ex, val| {
            Runtime::new(ex).r#return(val)
        })
    }

    /// Construct a Trilogy closure from a Rust closure.
    pub fn closure<F, const N: usize>(&mut self, cb: F) -> Value
    where
        F: FnMut(&mut Runtime, [Value; N]) -> crate::Result<()> + Sync + Send + 'static,
    {
        struct Callback<F, const N: usize>(F, PhantomData<[(); N]>);

        impl<F, const N: usize> NativeFunction for Callback<F, N>
        where
            F: FnMut(&mut Runtime, [Value; N]) -> crate::Result<()> + Sync + Send + 'static,
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
                    self.0(&mut Runtime::new(ex), args)
                } else {
                    // TODO: better error here... arity mismatch?
                    Err(ex.error(ErrorKind::RuntimeTypeError))
                }
            }
        }

        Value::from(Native::from(Callback(cb, PhantomData)))
    }
}
