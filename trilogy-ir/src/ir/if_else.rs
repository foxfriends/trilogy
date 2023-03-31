use super::*;
use crate::Analyzer;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct IfElse {
    pub condition: Expression,
    pub when_true: Expression,
    pub when_false: Expression,
}

impl IfElse {
    pub(super) fn convert(analyzer: &mut Analyzer, ast: syntax::IfStatement) -> Expression {
        let span = ast.span();
        let when_false = ast
            .if_false
            .map(|ast| Expression::convert_block(analyzer, ast))
            .unwrap_or_else(|| Expression::unit(span));
        ast.branches
            .into_iter()
            .rev()
            .fold(when_false, |when_false, branch| {
                let span = branch.span();
                let condition = Expression::convert(analyzer, branch.condition);
                let when_true = Expression::convert_block(analyzer, branch.body);
                Expression::if_else(
                    span,
                    Self {
                        condition,
                        when_true,
                        when_false,
                    },
                )
            })
    }
}
