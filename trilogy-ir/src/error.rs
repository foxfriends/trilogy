use crate::ir;
use source_span::Span;
use std::fmt::{self, Display};
use trilogy_parser::syntax;

#[derive(Debug)]
pub enum Error {
    UnknownExport {
        name: syntax::Identifier,
    },
    UnboundIdentifier {
        name: syntax::Identifier,
    },
    DuplicateDefinition {
        original: Span,
        duplicate: syntax::Identifier,
    },
    IdentifierInOwnDefinition {
        name: ir::Identifier,
    },
    AssignedImmutableBinding {
        name: ir::Identifier,
        assignment: Span,
    },
    DuplicateExport {
        original: Span,
        duplicate: syntax::Identifier,
    },
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnknownExport { .. } => write!(f, "unknown export"),
            Error::UnboundIdentifier { .. } => write!(f, "unbound identifier"),
            Error::DuplicateDefinition { .. } => write!(f, "duplicate definition"),
            Error::IdentifierInOwnDefinition { .. } => write!(f, "identifier in own definition"),
            Error::AssignedImmutableBinding { .. } => write!(f, "assigned immutable binding"),
            Error::DuplicateExport { .. } => write!(f, "duplicate export"),
        }
    }
}
