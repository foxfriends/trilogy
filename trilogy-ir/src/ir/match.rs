use super::*;
use crate::Converter;
use source_span::Span;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Match {
    pub expression: Expression,
    pub cases: Vec<Case>,
}

impl Match {
    pub(super) fn convert_statement(
        converter: &mut Converter,
        ast: syntax::MatchStatement,
    ) -> Expression {
        let span = ast.span();
        let expression = Expression::convert(converter, ast.expression);
        let else_case = ast
            .else_case
            .map(|ast| Case {
                span: ast.span(),
                pattern: ast
                    .pattern
                    .map(|ast| Expression::convert_binding(converter, ast))
                    .unwrap_or_else(|| Expression::wildcard(ast.r#else.span)),
                guard: Expression::boolean(ast.r#else.span, true),
                body: Expression::convert_block(converter, ast.body),
            })
            .unwrap_or_else(|| Case {
                span,
                pattern: Expression::wildcard(span),
                guard: Expression::boolean(span, true),
                body: Expression::unit(span),
            });
        let mut cases: Vec<_> = ast
            .cases
            .into_iter()
            .map(|ast| Case::convert_statement(converter, ast))
            .collect();
        cases.push(else_case);
        Expression::r#match(span, Self { expression, cases })
    }

    pub(super) fn convert_expression(
        converter: &mut Converter,
        ast: syntax::MatchExpression,
    ) -> Expression {
        let span = ast.span();
        let expression = Expression::convert(converter, ast.expression);
        let mut cases: Vec<_> = ast
            .cases
            .into_iter()
            .map(|ast| Case::convert_expression(converter, ast))
            .collect();
        cases.push(Case {
            span: ast.r#else.span().union(ast.no_match.span()),
            pattern: Expression::convert_pattern(converter, ast.else_binding),
            guard: Expression::boolean(ast.r#else.span, true),
            body: Expression::convert(converter, ast.no_match),
        });
        Expression::r#match(span, Self { expression, cases })
    }
}

#[derive(Clone, Debug)]
pub struct Case {
    pub span: Span,
    pub pattern: Expression,
    pub guard: Expression,
    pub body: Expression,
}

impl Case {
    fn convert_statement(converter: &mut Converter, ast: syntax::MatchStatementCase) -> Self {
        let case_span = ast.case.span;
        let span = ast.span();
        converter.push_scope();
        let pattern = ast
            .pattern
            .map(|ast| Expression::convert_pattern(converter, ast))
            .unwrap_or_else(|| Expression::wildcard(case_span));
        let guard = ast
            .guard
            .map(|ast| Expression::convert(converter, ast))
            .unwrap_or_else(|| Expression::boolean(case_span, true));
        let body = Expression::convert_block(converter, ast.body);
        converter.pop_scope();
        Self {
            span,
            pattern,
            guard,
            body,
        }
    }

    fn convert_expression(converter: &mut Converter, ast: syntax::MatchExpressionCase) -> Self {
        let case_span = ast.case.span;
        let span = ast.span();
        converter.push_scope();
        let pattern = ast
            .pattern
            .map(|ast| Expression::convert_pattern(converter, ast))
            .unwrap_or_else(|| Expression::wildcard(case_span));
        let guard = ast
            .guard
            .map(|ast| Expression::convert(converter, ast))
            .unwrap_or_else(|| Expression::boolean(case_span, true));
        let body = Expression::convert(converter, ast.body);
        converter.pop_scope();
        Self {
            span,
            pattern,
            guard,
            body,
        }
    }
}
