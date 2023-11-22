use super::Runtime;
use trilogy_vm::{Execution, Native, NativeFunction, Value};

#[derive(Clone)]
pub(super) struct CurriedCallback<F, const N: usize>(F, Vec<Value>);

impl<F, const N: usize> CurriedCallback<F, N> {
    pub fn new(f: F) -> Self {
        Self(f, vec![])
    }
}

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
