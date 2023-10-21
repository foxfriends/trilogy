use crate::ir;
use source_span::Span;
use trilogy_parser::syntax;

#[derive(Debug)]
pub enum Error {
    NoMainProcedure,
    MainNotProcedure {
        item: ir::DefinitionItem,
    },
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
}
