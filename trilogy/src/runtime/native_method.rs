use trilogy_vm::{Error, Execution, NativeFunction, Value};

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

pub trait NativeMethodFn: Send + Sync {
    type SelfType;

    fn arity(&self) -> usize;

    fn call(
        &mut self,
        receiver: &mut Self::SelfType,
        ex: &mut Execution,
        input: Vec<Value>,
    ) -> Result<(), Error>;
}

impl<T: Send + Sync, F: NativeMethodFn<SelfType = T>> NativeFunction for NativeMethod<T, F> {
    fn arity(&self) -> usize {
        NativeMethodFn::arity(&self.func)
    }

    fn call(&mut self, ex: &mut Execution, input: Vec<Value>) -> crate::Result<()> {
        self.func.call(&mut self.receiver, ex, input)
    }
}
