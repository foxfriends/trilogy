use crate::{Analyzer, Id};
use source_span::Span;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Identifier {
    span: Span,
    pub id: Id,
}

impl Identifier {
    pub(crate) fn declare(analyzer: &mut Analyzer, identifier: syntax::Identifier) -> Identifier {
        let span = identifier.span();
        let id = analyzer.declare(identifier.into());
        Self { span, id }
    }
}
