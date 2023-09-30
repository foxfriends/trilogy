use crate::bytecode::OpCodeError;
use std::error::Error;
use std::fmt::{self, Display};
use std::num::ParseIntError;

#[derive(Debug, Clone)]
pub struct AsmError {
    pub(super) position: usize,
    pub(super) kind: ErrorKind,
}

impl Error for AsmError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.kind {
            ErrorKind::Offset(err) => Some(err),
            ErrorKind::Opcode(err) => Some(err),
            _ => None,
        }
    }
}

impl Display for AsmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at position {}", self.kind, self.position)
    }
}

#[derive(Debug, Clone)]
pub(super) enum ErrorKind {
    Token,
    String,
    Label,
    Value,
    Opcode(OpCodeError),
    Offset(ParseIntError),
}

impl From<OpCodeError> for ErrorKind {
    fn from(value: OpCodeError) -> Self {
        Self::Opcode(value)
    }
}

impl From<ParseIntError> for ErrorKind {
    fn from(value: ParseIntError) -> Self {
        Self::Offset(value)
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String => write!(f, "incorrectly quoted string in label"),
            Self::Token => write!(f, "invalid or missing token"),
            Self::Label => write!(f, "invalid label"),
            Self::Value => write!(f, "invalid value"),
            Self::Opcode(err) => write!(f, "{err}"),
            Self::Offset(err) => write!(f, "invalid offset: {err}"),
        }
    }
}
