use super::*;
use crate::Converter;
use source_span::Span;
use trilogy_parser::{Spanned, syntax};

#[derive(Clone, Debug)]
pub struct Rule {
    pub span: Span,
    pub head_span: Span,
    pub parameters: Vec<Expression>,
    pub body: Query,
}

impl Rule {
    pub(super) fn convert(converter: &mut Converter, ast: syntax::RuleDefinition) -> Self {
        converter.push_scope();
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
        converter.pop_scope();
        Self {
            span,
            head_span,
            parameters,
            body,
        }
    }

    pub(super) fn convert_qy(converter: &mut Converter, ast: syntax::QyExpression) -> Self {
        converter.push_scope();
        let span = ast.span();
        let head_span = ast.qy.span.union(ast.close_paren.span);
        let parameters = ast
            .parameters
            .into_iter()
            .map(|param| Expression::convert_pattern(converter, param))
            .collect();
        let body = Query::convert(converter, ast.body);
        converter.pop_scope();
        Self {
            span,
            head_span,
            parameters,
            body,
        }
    }
}
