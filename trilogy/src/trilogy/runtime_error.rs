use std::fmt::{self, Display};

#[derive(Debug)]
pub struct RuntimeError {
    pub(super) error: trilogy_vm::Error,
}

impl std::error::Error for RuntimeError {}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}
