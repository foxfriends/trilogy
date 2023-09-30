use crate::bytecode::asm::AsmError;
use std::error::Error;
use std::fmt::{self, Display};

/// An error that can occur when building a [`Chunk`][crate::Chunk] incorrectly.
#[derive(Clone, Debug)]
pub enum ChunkError {
    /// A referenced label was not defined.
    MissingLabel(String),
    /// Parsed assembly string was invalid.
    InvalidAsm(AsmError),
}

impl Display for ChunkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingLabel(label) => write!(f, "label `{label}` was not defined"),
            Self::InvalidAsm(reason) => write!(f, "could not parse assembly due to {reason}"),
        }
    }
}

impl Error for ChunkError {
    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::InvalidAsm(error) => Some(error),
            _ => None,
        }
    }
}
