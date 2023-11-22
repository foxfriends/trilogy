use super::Runtime;
use trilogy_vm::{Execution, NativeFunction, Value};

pub(super) struct Callback<F, const N: usize>(F);

impl<F, const N: usize> Callback<F, N> {
    pub fn new(f: F) -> Self {
        Self(f)
    }
}

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
