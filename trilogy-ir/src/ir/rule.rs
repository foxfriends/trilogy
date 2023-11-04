use super::*;
use crate::Converter;
use source_span::Span;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Rule {
    pub span: Span,
    pub head_span: Span,
    pub parameters: Vec<Expression>,
    pub body: Query,
}

impl Rule {
    pub(super) fn convert(converter: &mut Converter, ast: syntax::RuleDefinition) -> Self {
        let span = ast.span();
        let head_span = ast.head.span();
        let parameters = ast
            .head
            .parameters
            .into_iter()
            .map(|param| Expression::convert_pattern(converter, param))
            .collect();
        let body = ast
            .body
            .map(|query| Query::convert(converter, query))
            .unwrap_or_else(|| Query::pass(span));
        Self {
            span,
            head_span,
            parameters,
            body,
        }
    }
}
