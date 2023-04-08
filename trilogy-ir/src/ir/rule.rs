use super::*;
use crate::Analyzer;
use source_span::Span;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Rule {
    pub span: Span,
    pub parameters: Vec<Expression>,
    pub body: Expression,
}

impl Rule {
    pub(super) fn convert(analyzer: &mut Analyzer, ast: syntax::RuleDefinition) -> Self {
        let span = ast.span();
        let parameters = ast
            .head
            .parameters
            .into_iter()
            .map(|param| Expression::convert_pattern(analyzer, param))
            .collect();
        let body = ast
            .body
            .map(|query| Expression::convert_query(analyzer, query))
            .unwrap_or_else(|| Expression::query(span, Query::pass(span)));
        Self {
            span,
            parameters,
            body,
        }
    }
}
