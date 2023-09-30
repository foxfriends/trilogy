use std::fmt::{self, Display};

#[derive(Debug)]
pub struct RuntimeError {
    pub(super) error: trilogy_vm::Error,
}

impl From<trilogy_vm::Error> for RuntimeError {
    fn from(error: trilogy_vm::Error) -> Self {
        Self { error }
    }
}

impl std::error::Error for RuntimeError {}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.error)?;
        writeln!(f, "Final IP: {}", self.error.ip)?;
        write!(f, "Stack Dump:\n{}", self.error.dump())?;
        Ok(())
    }
}
