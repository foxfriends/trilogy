mod callback;
mod curried_callback;
mod native_method;
mod native_module;
mod runtime_error;

pub use native_method::*;
pub use native_module::*;
pub use runtime_error::*;

use callback::Callback;
use curried_callback::CurriedCallback;
use trilogy_codegen::YIELD;
use trilogy_vm::{Atom, ErrorKind, InternalRuntimeError, Native, Struct, Tuple, Value};

/// A handle to the Trilogy language ex, allowing native functions written
/// in Rust to interact effectively with the running Trilogy program.
///
/// Due to Trilogy's control flow being more flexible than Rust's, we
/// cannot effectively use Rust control flow to manipulate a Trilogy
/// program. Instead, control flow is done in continuation passing style
/// using specific continuations provided by the ex.
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

    pub fn r#struct<V: Into<Value>>(&self, tag: &str, value: V) -> Struct {
        Struct::new(self.0.atom(tag), value.into())
    }

    pub fn atom_anon(&self, tag: &str) -> Atom {
        self.0.atom_anon(tag)
    }

    pub fn runtime_error<V: Into<Value>>(&self, value: V) -> trilogy_vm::Error {
        self.0.error(ErrorKind::RuntimeError(value.into()))
    }

    pub fn runtime_type_error<V: Into<Value>>(&self, value: V) -> trilogy_vm::Error {
        self.runtime_error(self.r#struct("RuntimeTypeError", value))
    }

    pub fn incorrect_arity(&self, arity: usize) -> trilogy_vm::Error {
        self.runtime_error(self.r#struct("IncorrectArity", arity))
    }

    pub fn unresolved_import(&self, atom: Atom, symbols: Vec<Value>) -> trilogy_vm::Error {
        self.runtime_error(
            self.r#struct("UnresolvedImport", Tuple::new(atom.into(), symbols.into())),
        )
    }

    pub fn invalid_call(&self, call_type: Atom) -> trilogy_vm::Error {
        self.runtime_error(self.r#struct("InvalidCall", call_type))
    }

    fn unlock(&self, call_type: &str, arity: usize, args: &[Value]) -> crate::Result<()> {
        match args.last() {
            Some(Value::Struct(s))
                if s.name() == self.atom(call_type) && *s.value() == Value::from(arity) =>
            {
                Ok(())
            }
            Some(Value::Struct(s)) if s.name() == self.atom(call_type) => {
                Err(self.incorrect_arity(1))
            }
            Some(Value::Struct(s)) => Err(self.invalid_call(s.name())),
            _ => Err(self.0.error(ErrorKind::InternalRuntimeError(
                InternalRuntimeError::TypeError,
            ))),
        }
    }

    /// Parses the arguments of a call as a Trilogy procedure does, allowing a `NativeFunction`
    /// to act as a procedure.
    pub fn unlock_procedure<const N: usize>(
        &self,
        mut args: Vec<Value>,
    ) -> crate::Result<[Value; N]> {
        self.unlock("procedure", N, &args)?;
        args.pop();
        Ok(args.try_into().unwrap())
    }

    /// Parses the arguments of a call as a Trilogy function does, allowing a `NativeFunction`
    /// to act as a function.
    pub fn unlock_function(&self, mut args: Vec<Value>) -> crate::Result<Value> {
        self.unlock("function", 1, &args)?;
        args.pop();
        Ok(args.pop().unwrap())
    }

    pub fn unlock_module(&self, mut args: Vec<Value>) -> crate::Result<Atom> {
        self.unlock("module", 1, &args)?;
        args.pop();
        match args.pop().unwrap() {
            Value::Atom(atom) => Ok(atom),
            _ => Err(self.0.error(ErrorKind::InternalRuntimeError(
                InternalRuntimeError::TypeError,
            ))),
        }
    }

    /// The equivalent of the yield operator, allowing a native function to
    /// yield an effect.
    pub fn r#yield<F>(self, value: impl Into<Value>, mut f: F) -> crate::Result<()>
    where
        F: FnMut(Runtime, Value) -> crate::Result<()> + Sync + Send + 'static,
    {
        let function = self.0.atom("function");
        let key = Struct::new(function, 1);
        let y = self.0.procedure(YIELD)?;
        let effect = value.into();
        self.0
            .callback(y, vec![effect, key.into()], move |ex, val| {
                f(Runtime::new(ex), val)
            })
    }

    /// The equivalent of the return operator, allowing a native function to return a value.
    ///
    /// Calling return more than once is not permitted.
    pub fn r#return(self, value: impl Into<Value>) -> crate::Result<()> {
        self.0.r#return(value.into())
    }

    /// Construct a Trilogy procedure closure (e.g. `do()`) from a Rust closure.
    pub fn procedure_closure<F, const N: usize>(&self, cb: F) -> Value
    where
        F: FnMut(Runtime, [Value; N]) -> crate::Result<()> + Sync + Send + 'static,
    {
        Value::from(Native::from(Callback::new(cb)))
    }

    /// Construct a Trilogy function closure (e.g. `fn()`) from a Rust closure.
    pub fn function_closure<F, const N: usize>(&self, cb: F) -> Value
    where
        F: FnMut(Runtime, [Value; N]) -> crate::Result<()> + Sync + Send + Clone + 'static,
    {
        Value::from(Native::from(CurriedCallback::<F, N>::new(cb)))
    }

    /// Apply a value that is expected to be a function.
    ///
    /// This is equivalent to the Trilogy expression `function argument`.
    pub fn apply_function<F>(self, function: Value, argument: Value, mut f: F) -> crate::Result<()>
    where
        F: FnMut(Runtime, Value) -> crate::Result<()> + Sync + Send + 'static,
    {
        match function {
            Value::Callable(..) => {
                let key = Struct::new(self.atom("function"), 1);
                self.0
                    .callback(function, vec![argument, key.into()], move |ex, val| {
                        f(Runtime::new(ex), val)
                    })
            }
            value => Err(self.runtime_type_error(value)),
        }
    }

    pub fn typecheck<T>(&self, value: Value) -> crate::Result<T>
    where
        Value: TryInto<T, Error = Value>,
    {
        value
            .try_into()
            .map_err(|value| self.runtime_type_error(value))
    }
}
