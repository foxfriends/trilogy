use std::fmt::{self, Display};
use trilogy_vm::Program;

#[derive(Debug)]
pub struct RuntimeError<'a> {
    pub(super) program: &'a Program,
    pub(super) error: trilogy_vm::Error,
}

impl std::error::Error for RuntimeError<'_> {}

impl Display for RuntimeError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Trace:\n{}", self.error.trace(self.program))?;
        write!(f, "Dump:\n{}", self.error.dump())?;
        write!(f, "{}", self.error)?;
        Ok(())
    }
}
