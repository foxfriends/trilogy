use std::fmt::{self, Display};
use trilogy_ir::ir;

#[derive(Debug)]
pub enum ErrorKind {
    NoMainProcedure,
    MainHasParameters { proc: ir::ProcedureDefinition },
    MainNotProcedure { item: ir::DefinitionItem },
}

impl std::error::Error for ErrorKind {}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoMainProcedure => write!(f, "no main procedure"),
            Self::MainHasParameters { .. } => write!(f, "main has parameters"),
            Self::MainNotProcedure { .. } => write!(f, "main not procedure"),
        }
    }
}
