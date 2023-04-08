use super::*;
use crate::Analyzer;
use trilogy_parser::syntax;

#[derive(Clone, Debug)]
pub struct Unification {
    pub pattern: Expression,
    pub expression: Expression,
}

impl Unification {
    pub(super) fn new(pattern: Expression, expression: Expression) -> Self {
        Self {
            pattern,
            expression,
        }
    }

    pub(super) fn convert_direct(analyzer: &mut Analyzer, ast: syntax::DirectUnification) -> Self {
        Self::convert(analyzer, ast.pattern, ast.expression)
    }

    pub(super) fn convert_element(
        analyzer: &mut Analyzer,
        ast: syntax::ElementUnification,
    ) -> Self {
        Self::convert(analyzer, ast.pattern, ast.expression)
    }

    fn convert(
        analyzer: &mut Analyzer,
        pattern: syntax::Pattern,
        expression: syntax::Expression,
    ) -> Self {
        let pattern = Expression::convert_pattern(analyzer, pattern);
        let expression = Expression::convert(analyzer, expression);
        Self::new(pattern, expression)
    }
}
