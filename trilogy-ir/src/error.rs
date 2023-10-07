use crate::ir;
use source_span::Span;
use trilogy_parser::syntax;

#[derive(Debug)]
pub enum Error {
    UnknownExport { name: syntax::Identifier },
    UnboundIdentifier { name: syntax::Identifier },
    UnknownModule { name: syntax::Identifier },
    DuplicateDefinition { name: syntax::Identifier },
    IdentifierInOwnDefinition { name: ir::Identifier },
    DisjointBindings { span: Span },
    AssignedImmutableBinding { name: ir::Identifier },
}
