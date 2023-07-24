use super::*;
use crate::Analyzer;
use source_span::Span;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Procedure {
    pub span: Span,
    pub parameters: Vec<Expression>,
    pub body: Expression,
}

impl Procedure {
    pub(super) fn convert(analyzer: &mut Analyzer, ast: syntax::ProcedureDefinition) -> Self {
        let span = ast.span();
        let parameters: Vec<_> = ast
            .head
            .parameters
            .into_iter()
            .map(|param| Expression::convert_pattern(analyzer, param))
            .collect();
        let body = Expression::convert_block(analyzer, ast.body);
        Self {
            span,
            parameters,
            body,
        }
    }

    pub(super) fn convert_do(analyzer: &mut Analyzer, ast: syntax::DoExpression) -> Self {
        let span = ast.span();
        let do_span = ast.do_token().span();
        let parameters: Vec<_> = ast
            .parameters
            .into_iter()
            .map(|param| Expression::convert_pattern(analyzer, param))
            .collect();
        let body = match ast.body {
            syntax::DoBody::Block(ast) => Expression::convert_block(analyzer, *ast),
            syntax::DoBody::Expression(expr) => Expression::builtin(do_span, Builtin::Return)
                .apply_to(span, Expression::convert(analyzer, *expr)),
        };
        Self {
            span,
            parameters,
            body,
        }
    }
}
