use std::error::Error;
use std::fmt::{self, Display};

#[derive(Clone, Debug)]
pub struct AsmError {
    pub line: usize,
    pub error: ErrorKind,
}

#[derive(Clone, Debug)]
pub enum ErrorKind {
    UnknownOpcode(String),
    MissingParameter,
    InvalidOffset,
    InvalidLabelReference,
    MissingLabel(String),
    InvalidValue(ValueError),
}

impl Error for AsmError {}

impl Display for AsmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid asm on line {}: {}", self.line, self.error)
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownOpcode(opcode) => write!(f, "unknown opcode {opcode}"),
            Self::InvalidLabelReference => write!(f, "improperly formatted label reference"),
            Self::InvalidOffset => write!(f, "improperly formatted offset"),
            Self::MissingParameter => write!(f, "missing parameter"),
            Self::MissingLabel(label) => write!(f, "label `{label}` not defined"),
            Self::InvalidValue(error) => error.fmt(f),
        }
    }
}

impl From<ValueError> for ErrorKind {
    fn from(value: ValueError) -> Self {
        Self::InvalidValue(value)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ValueError {
    InvalidProcedure,
    InvalidCharacter,
    InvalidAtom,
    InvalidTuple,
    InvalidNumber,
    InvalidStruct,
    InvalidArray,
    InvalidString,
    InvalidRecord,
    InvalidSet,
    ExtraChars,
}

impl Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProcedure => write!(f, "improperly formatted procedure reference"),
            Self::InvalidCharacter => write!(f, "improperly formatted character"),
            Self::InvalidAtom => write!(f, "improperly formatted atom"),
            Self::InvalidTuple => write!(f, "improperly formatted tuple"),
            Self::InvalidNumber => write!(f, "improperly formatted number"),
            Self::InvalidStruct => write!(f, "improperly formatted struct"),
            Self::InvalidArray => write!(f, "improperly formatted array"),
            Self::InvalidString => write!(f, "improperly formatted string"),
            Self::InvalidRecord => write!(f, "improperly formatted record"),
            Self::InvalidSet => write!(f, "improperly formatted set"),
            Self::ExtraChars => write!(f, "extra characters at end of line"),
        }
    }
}
