use super::*;
use crate::Analyzer;
use source_span::Span;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Match {
    pub expression: Expression,
    pub cases: Vec<Case>,
}

impl Match {
    pub(super) fn convert(analyzer: &mut Analyzer, ast: syntax::MatchStatement) -> Expression {
        let span = ast.span();
        let expression = Expression::convert(analyzer, ast.expression);
        let cases = ast
            .cases
            .into_iter()
            .map(|ast| Case::convert(analyzer, ast))
            .collect();
        Expression::r#match(span, Self { expression, cases })
    }
}

#[derive(Clone, Debug)]
pub struct Case {
    span: Span,
    pub pattern: Pattern,
    pub guard: Expression,
    pub body: Expression,
}

impl Case {
    fn convert(analyzer: &mut Analyzer, ast: syntax::MatchStatementCase) -> Self {
        let case_span = ast.case_token().span;
        let span = ast.span();
        let pattern = ast
            .pattern
            .map(|ast| Pattern::convert(analyzer, ast))
            .unwrap_or_else(|| Pattern::wildcard(case_span));
        let guard = ast
            .guard
            .map(|ast| Expression::convert(analyzer, ast))
            .unwrap_or_else(|| Expression::boolean(case_span, true));
        let body = Expression::convert_block(analyzer, ast.body);
        Self {
            span,
            pattern,
            guard,
            body,
        }
    }
}
