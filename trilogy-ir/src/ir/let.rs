use super::*;
use crate::Converter;
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
        converter: &mut Converter,
        ast: syntax::LetStatement,
        rest: &mut impl std::iter::Iterator<Item = syntax::Statement>,
    ) -> Expression {
        let span = ast.span();
        let query = Query::convert(converter, ast.query);
        let body = match Expression::convert_sequence(converter, rest) {
            Some(body) => {
                let body_span = body
                    .iter()
                    .map(|expr| expr.span)
                    .reduce(|a, b| a.union(b))
                    .unwrap();
                Expression::sequence(body_span, body)
            }
            None => Expression::unit(span),
        };
        Expression::r#let(span, crate::ir::Let::new(query, body))
    }

    pub(super) fn convert(converter: &mut Converter, ast: syntax::LetExpression) -> Expression {
        let span = ast.span();
        let query = Query::convert(converter, ast.query);
        let body = Expression::convert(converter, ast.body);
        Expression::r#let(span, Self::new(query, body))
    }
}
