use super::*;
use crate::Converter;
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
    pub(super) fn convert(converter: &mut Converter, ast: syntax::FunctionDefinition) -> Self {
        converter.push_scope();
        let span = ast.span();
        let head_span = ast.head.span();
        let parameters: Vec<_> = ast
            .head
            .parameters
            .into_iter()
            .map(|param| Expression::convert_pattern(converter, param))
            .collect();
        let body = Expression::builtin(span, Builtin::Return)
            .apply_to(span, Expression::convert(converter, ast.body));
        converter.pop_scope();
        Self {
            span,
            head_span,
            parameters,
            body,
        }
    }

    pub(super) fn convert_fn(converter: &mut Converter, ast: syntax::FnExpression) -> Self {
        converter.push_scope();
        let span = ast.span();
        let parameters: Vec<_> = ast
            .parameters
            .into_iter()
            .map(|param| Expression::convert_pattern(converter, param))
            .collect();
        let body = Expression::builtin(span, Builtin::Return)
            .apply_to(span, Expression::convert(converter, ast.body));
        converter.pop_scope();
        Self {
            head_span: ast.r#fn.span.union(ast.dot.span),
            span,
            parameters,
            body,
        }
    }
}
