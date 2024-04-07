use trilogy_vm::{Error, Execution, NativeFunction, Value};

/// Wraps a Rust method for use as a Trilogy native function.
///
/// These are not typically created manually, but as part of the
/// [`module`][trilogy_derive::module] proc macro when used on a
/// Rust `impl` block.
#[derive(Clone)]
pub struct NativeMethod<T, F> {
    receiver: T,
    func: F,
}

impl<T, F> NativeMethod<T, F> {
    pub fn new(receiver: T, func: F) -> Self {
        Self { receiver, func }
    }
}

pub trait NativeMethodFn: trilogy_vm::Threading {
    type SelfType;

    fn arity(&self) -> usize;

    fn call(
        &mut self,
        receiver: &mut Self::SelfType,
        ex: &mut Execution,
        input: Vec<Value>,
    ) -> Result<(), Error>;
}

impl<T: trilogy_vm::Threading + 'static, F: NativeMethodFn<SelfType = T> + 'static> NativeFunction
    for NativeMethod<T, F>
{
    fn arity(&self) -> usize {
        NativeMethodFn::arity(&self.func)
    }

    fn call(&mut self, ex: &mut Execution, input: Vec<Value>) -> crate::Result<()> {
        self.func.call(&mut self.receiver, ex, input)
    }
}
