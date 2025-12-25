use super::*;
use crate::Converter;
use source_span::Span;
use trilogy_parser::{
    Spanned,
    syntax::{self, FollowingExpression},
};

#[derive(Clone, Debug)]
pub struct Match {
    pub expression: Expression,
    pub cases: Vec<Case>,
}

impl Match {
    pub(super) fn convert_statement(
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
        match ast.else_case {
            Some(ast) => {
                let span = ast.r#else.span().union(ast.body.span());
                converter.push_scope();
                let pattern = Expression::wildcard(ast.r#else.span);
                let guard = Expression::boolean(ast.r#else.span, true);
                let body = Expression::convert(converter, ast.body);
                converter.pop_scope();
                cases.push(Case {
                    span,
                    pattern,
                    guard,
                    body,
                });
            }
            None => {
                cases.push(Case {
                    span,
                    pattern: Expression::wildcard(span),
                    guard: Expression::boolean(span, true),
                    body: Expression::unit(span),
                });
            }
        }
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
        match ast.else_case {
            Some(ast) => {
                let span = ast.r#else.span().union(ast.body.span());
                converter.push_scope();
                let pattern = Expression::wildcard(ast.r#else.span);
                let guard = Expression::boolean(ast.r#else.span, true);
                let body = Expression::convert(converter, ast.body);
                converter.pop_scope();
                cases.push(Case {
                    span,
                    pattern,
                    guard,
                    body,
                });
            }
            None => {
                cases.push(Case {
                    span,
                    pattern: Expression::wildcard(span),
                    guard: Expression::boolean(span, true),
                    body: Expression::end(span),
                });
            }
        }
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
            .map(|ast| Expression::convert(converter, ast.expression))
            .unwrap_or_else(|| Expression::boolean(case_span, true));
        let body = match ast.body {
            FollowingExpression::Then(_, body) => Expression::convert(converter, body),
            FollowingExpression::Block(body) => Expression::convert_block(converter, body),
        };
        converter.pop_scope();
        Self {
            span,
            pattern,
            guard,
            body,
        }
    }
}
