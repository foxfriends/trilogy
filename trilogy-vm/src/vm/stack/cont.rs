use crate::{Error, Execution, Offset, Value};
use std::fmt::{self, Debug};
use std::sync::{Arc, Mutex};

#[allow(clippy::type_complexity)]
#[derive(Clone)]
pub(crate) struct Callback(
    Arc<Mutex<dyn FnMut(&mut Execution, Value) -> Result<(), Error> + Sync + Send + 'static>>,
);

impl Callback {
    pub fn call(&self, ex: &mut Execution, value: Value) -> Result<(), Error> {
        let mut callback = self.0.lock().unwrap();
        callback(ex, value)
    }
}

#[derive(Clone)]
pub(crate) enum Cont {
    Offset(Offset),
    Callback(Callback),
}

impl Debug for Cont {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Offset(ip) => write!(f, "{ip}"),
            Self::Callback(..) => write!(f, "rust"),
        }
    }
}

impl<F> From<F> for Cont
where
    F: FnMut(&mut Execution, Value) -> Result<(), Error> + Sync + Send + 'static,
{
    fn from(value: F) -> Self {
        Cont::Callback(Callback(Arc::new(Mutex::new(value))))
    }
}

impl From<Offset> for Cont {
    fn from(value: Offset) -> Self {
        Cont::Offset(value)
    }
}
