use super::*;
use crate::Converter;
use source_span::Span;
use trilogy_parser::{Spanned, syntax};

#[derive(Clone, Debug)]
pub struct Procedure {
    pub span: Span,
    pub head_span: Span,
    pub parameters: Vec<Expression>,
    pub body: Expression,
}

impl Procedure {
    pub(super) fn convert(converter: &mut Converter, ast: syntax::ProcedureDefinition) -> Self {
        converter.push_scope();
        let span = ast.span();
        let head_span = ast.head.span();
        let parameters: Vec<_> = ast
            .head
            .parameters
            .into_iter()
            .map(|param| Expression::convert_pattern(converter, param))
            .collect();
        let body = Expression::convert_block(converter, ast.body);
        converter.pop_scope();
        Self {
            span,
            head_span,
            parameters,
            body,
        }
    }

    pub(super) fn convert_do(converter: &mut Converter, ast: syntax::DoExpression) -> Self {
        converter.push_scope();
        let span = ast.span();
        let do_span = ast.do_token().span();
        let parameters: Vec<_> = ast
            .parameters
            .into_iter()
            .map(|param| Expression::convert_pattern(converter, param))
            .collect();
        let body = match ast.body {
            syntax::DoBody::Block(ast) => Expression::convert_block(converter, *ast),
            syntax::DoBody::Expression(expr) => Expression::builtin(do_span, Builtin::Return)
                .apply_to(span, Expression::convert(converter, *expr)),
        };
        converter.pop_scope();
        Self {
            span,
            head_span: do_span.union(ast.close_paren.span),
            parameters,
            body,
        }
    }
}
