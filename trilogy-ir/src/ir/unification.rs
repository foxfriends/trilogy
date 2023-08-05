use super::*;
use crate::visitor::{Bindings, Identifiers};
use crate::{Analyzer, Error};
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

        let unification = Self::new(pattern, expression);
        let violations = validate_unification(&unification);
        for violation in violations {
            analyzer.error(Error::IdentifierInOwnDefinition { name: violation });
        }
        unification
    }
}

fn validate_unification(unification: &Unification) -> Vec<Identifier> {
    let declared_ids = Bindings::of(&unification.pattern);
    let used_ids = Identifiers::of(&unification.expression);
    used_ids
        .into_iter()
        .filter(|ident| declared_ids.contains(&ident.id))
        .collect::<Vec<_>>()
}
