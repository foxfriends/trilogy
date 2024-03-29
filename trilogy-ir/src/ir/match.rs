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
            .map(|ast| Expression::convert_block(converter, ast))
            .unwrap_or_else(|| Expression::unit(span));
        let mut cases: Vec<_> = ast
            .cases
            .into_iter()
            .map(|ast| Case::convert_statement(converter, ast))
            .collect();
        cases.push(Case::new_fallback(else_case));
        Expression::r#match(span, Self { expression, cases })
    }

    pub(super) fn convert_expression(
        converter: &mut Converter,
        ast: syntax::MatchExpression,
    ) -> Expression {
        let span = ast.span();
        let expression = Expression::convert(converter, ast.expression);
        let else_case = Expression::convert(converter, ast.no_match);
        let mut cases: Vec<_> = ast
            .cases
            .into_iter()
            .map(|ast| Case::convert_expression(converter, ast))
            .collect();
        cases.push(Case::new_fallback(else_case));
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
    fn new_fallback(body: Expression) -> Self {
        Self {
            span: body.span,
            // TODO: would be nice to have the `else` span here
            pattern: Expression::wildcard(body.span),
            guard: Expression::boolean(body.span, true),
            body,
        }
    }

    fn convert_statement(converter: &mut Converter, ast: syntax::MatchStatementCase) -> Self {
        let case_span = ast.case_token().span;
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
        let case_span = ast.case_token().span;
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
