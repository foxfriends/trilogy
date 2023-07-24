use super::*;
use crate::Analyzer;
use source_span::Span;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Function {
    pub span: Span,
    pub parameters: Vec<Expression>,
    pub body: Expression,
}

impl Function {
    pub(super) fn convert(analyzer: &mut Analyzer, ast: syntax::FunctionDefinition) -> Self {
        let span = ast.span();
        let parameters: Vec<_> = ast
            .head
            .parameters
            .into_iter()
            .map(|param| Expression::convert_pattern(analyzer, param))
            .collect();
        let body = Expression::convert(analyzer, ast.body);
        Self {
            span,
            parameters,
            body,
        }
    }

    pub(super) fn convert_fn(analyzer: &mut Analyzer, ast: syntax::FnExpression) -> Self {
        let span = ast.span();
        let parameters: Vec<_> = ast
            .parameters
            .into_iter()
            .map(|param| Expression::convert_pattern(analyzer, param))
            .collect();
        let body = Expression::convert(analyzer, ast.body);
        Self {
            span,
            parameters,
            body,
        }
    }
}
