use super::*;
use crate::{Analyzer, Id};
use source_span::Span;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Let {
    pub query: Query,
    pub body: Expression,
}

impl Let {
    pub(super) fn new(query: Query, body: Expression) -> Self {
        Self { query, body }
    }

    pub(super) fn convert_statement(
        analyzer: &mut Analyzer,
        ast: syntax::LetStatement,
        rest: &mut impl std::iter::Iterator<Item = syntax::Statement>,
    ) -> Expression {
        let span = ast.span();
        let query = Query::convert(analyzer, ast.query);
        let body = Expression::convert_sequence(analyzer, rest);
        // TODO: Span::default() is not best here, but there's not really a proper span for
        // this, so what to do?
        Expression::r#let(
            span,
            crate::ir::Let::new(query, Expression::sequence(Span::default(), body)),
        )
    }

    pub(super) fn convert(analyzer: &mut Analyzer, ast: syntax::LetExpression) -> Expression {
        let span = ast.span();
        let query = Query::convert(analyzer, ast.query);
        let body = Expression::convert(analyzer, ast.body);
        Expression::r#let(span, Self::new(query, body))
    }

    pub fn bindings(&self) -> impl std::iter::Iterator<Item = Id> + '_ {
        self.query.bindings().chain(self.body.bindings())
    }
}
