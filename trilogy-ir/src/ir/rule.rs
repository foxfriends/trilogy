use super::*;
use crate::Analyzer;
use source_span::Span;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Rule {
    pub span: Span,
    pub parameters: Vec<Expression>,
    pub body: Query,
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
            .map(|query| Query::convert(analyzer, query))
            .unwrap_or_else(|| Query::pass(span));
        Self {
            span,
            parameters,
            body,
        }
    }
}
