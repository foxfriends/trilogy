use super::*;
use crate::Analyzer;
use source_span::Span;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Function {
    pub span: Span,
    pub head_span: Span,
    pub parameters: Vec<Expression>,
    pub body: Expression,
}

impl Function {
    pub(super) fn convert(analyzer: &mut Analyzer, ast: syntax::FunctionDefinition) -> Self {
        let span = ast.span();
        let head_span = ast.head.span();
        let parameters: Vec<_> = ast
            .head
            .parameters
            .into_iter()
            .map(|param| Expression::convert_pattern(analyzer, param))
            .collect();
        let body = Expression::builtin(span, Builtin::Return)
            .apply_to(span, Expression::convert(analyzer, ast.body));
        Self {
            span,
            head_span,
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
        let body = Expression::builtin(span, Builtin::Return)
            .apply_to(span, Expression::convert(analyzer, ast.body));
        Self {
            head_span: ast.r#fn.span.union(ast.dot.span),
            span,
            parameters,
            body,
        }
    }
}
