use trilogy_codegen::YIELD;
use trilogy_vm::{
    Atom, ErrorKind, Execution, InternalRuntimeError, Native, NativeFunction, Struct, Tuple, Value,
};

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

    pub fn atom_anon(&self, tag: &str) -> Atom {
        self.0.atom_anon(tag)
    }

    pub fn runtime_type_error(&self, value: Value) -> trilogy_vm::Error {
        let atom = self.atom("RuntimeTypeError");
        let value = Struct::new(atom, value).into();
        self.0.error(ErrorKind::RuntimeError(value))
    }

    pub fn incorrect_arity(&self, arity: usize) -> trilogy_vm::Error {
        let atom = self.atom("IncorrectArity");
        let err_value = Struct::new(atom, arity);
        self.0.error(ErrorKind::RuntimeError(err_value.into()))
    }

    pub fn unresolved_import(&self, atom: Atom, symbols: Vec<Value>) -> trilogy_vm::Error {
        let err_value = Struct::new(
            self.atom("UnresolvedImport"),
            Tuple::new(atom.into(), symbols.into()),
        );
        self.0.error(ErrorKind::RuntimeError(err_value.into()))
    }

    pub fn invalid_call(&self, call_type: Atom) -> trilogy_vm::Error {
        let atom = self.atom("InvalidCall");
        let err_value = Struct::new(atom, call_type);
        self.0.error(ErrorKind::RuntimeError(err_value.into()))
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
        struct Callback<F, const N: usize>(F);

        impl<F, const N: usize> NativeFunction for Callback<F, N>
        where
            F: FnMut(Runtime, [Value; N]) -> crate::Result<()> + Sync + Send + 'static,
        {
            fn arity(&self) -> usize {
                N
            }

            fn call(&mut self, ex: &mut Execution, input: Vec<Value>) -> crate::Result<()> {
                let runtime = Runtime::new(ex);
                let input = runtime.unlock_procedure(input)?;
                self.0(runtime, input)
            }
        }

        Value::from(Native::from(Callback(cb)))
    }

    /// Construct a Trilogy function closure (e.g. `fn()`) from a Rust closure.
    pub fn function_closure<F, const N: usize>(&self, cb: F) -> Value
    where
        F: FnMut(Runtime, [Value; N]) -> crate::Result<()> + Sync + Send + Clone + 'static,
    {
        Value::from(Native::from(CurriedCallback::<F, N>(cb, vec![])))
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
}

#[derive(Clone)]
struct CurriedCallback<F, const N: usize>(F, Vec<Value>);

impl<F, const N: usize> NativeFunction for CurriedCallback<F, N>
where
    F: FnMut(Runtime, [Value; N]) -> crate::Result<()> + Sync + Send + Clone + 'static,
{
    fn arity(&self) -> usize {
        2
    }

    fn call(&mut self, ex: &mut Execution, input: Vec<Value>) -> Result<(), trilogy_vm::Error> {
        let runtime = Runtime::new(ex);
        let input = runtime.unlock_function(input)?;
        let mut next = self.clone();
        next.1.push(input);
        if next.1.len() == N {
            next.0(runtime, next.1.try_into().unwrap())
        } else {
            runtime.r#return(Native::from(next))
        }
    }
}
