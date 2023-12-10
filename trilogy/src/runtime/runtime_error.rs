use std::fmt::{self, Debug, Display};

/// A black box of failure that occurred during the execution of a Trilogy program.
///
/// Such an error might be a runtime error thrown by the program being executed, or
/// an error that occurred within the virtual machine itself, likely from attempting
/// to run invalid bytecode.
///
/// Language runtime errors may be unwrapped and inspected, but internal errors are
/// inaccessible.
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

        writeln!(f, "Stack trace:")?;
        for (i, frame) in self.error.stack_trace.frames.iter().enumerate() {
            writeln!(f, "{i}:")?;
            for (label, location) in &frame.annotations {
                writeln!(f, "\t{} ({})", label, location)?;
            }
        }

        Ok(())
    }
}

impl Debug for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.error)?;
        writeln!(f, "Final IP: {}", self.error.ip)?;
        write!(f, "Stack Dump:\n{}", self.error.dump())?;
        Ok(())
    }
}
