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
        let head_span = ast.head.span();
        let parameters: Vec<_> = ast
            .head
            .parameter_list
            .expect("do expressions require parameter lists, even empty")
            .parameters
            .into_iter()
            .map(|param| Expression::convert_pattern(converter, param))
            .collect();
        let body = match ast.body {
            syntax::DoBody::Block(ast) => Expression::convert_block(converter, *ast),
            syntax::DoBody::Expression(expr) => {
                Expression::builtin(ast.head.r#do.span, Builtin::Return)
                    .apply_to(span, Expression::convert(converter, *expr))
            }
        };
        converter.pop_scope();
        Self {
            span,
            head_span,
            parameters,
            body,
        }
    }

    pub(super) fn convert_using(
        converter: &mut Converter,
        using_span: Span,
        head: Option<syntax::DoHead>,
        body: &mut impl std::iter::Iterator<Item = syntax::Statement>,
    ) -> Self {
        converter.push_scope();
        let head_span = head.as_ref().map(|head| head.span()).unwrap_or(using_span);
        let parameters: Vec<_> = head
            .and_then(|head| head.parameter_list)
            .into_iter()
            .flat_map(|pl| pl.parameters)
            .map(|param| Expression::convert_pattern(converter, param))
            .collect();
        let body = Expression::convert_sequence(converter, body)
            .map(|seq| Expression::sequence(using_span.union(seq.last().unwrap().span), seq))
            .unwrap_or(Expression::unit(using_span));
        let span = body.span.union(head_span);
        let body = Expression::builtin(using_span, Builtin::Return).apply_to(span, body);
        converter.pop_scope();
        Self {
            span: body.span.union(head_span),
            head_span,
            parameters,
            body,
        }
    }
}
