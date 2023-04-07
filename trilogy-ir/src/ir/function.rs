use super::*;
use crate::Analyzer;
use source_span::Span;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Function {
    span: Span,
    pub parameters: Vec<Pattern>,
    pub body: Expression,
}

impl Function {
    pub(super) fn convert(analyzer: &mut Analyzer, ast: syntax::FunctionDefinition) -> Self {
        let span = ast.span();
        let parameters = ast
            .head
            .parameters
            .into_iter()
            .map(|param| Pattern::convert(analyzer, param))
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
        let parameters = ast
            .parameters
            .into_iter()
            .map(|param| Pattern::convert(analyzer, param))
            .collect();
        let body = Expression::convert(analyzer, ast.body);
        Self {
            span,
            parameters,
            body,
        }
    }
}
