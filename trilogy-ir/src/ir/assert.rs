use super::*;
use crate::Analyzer;
use trilogy_parser::syntax;

#[derive(Clone, Debug)]
pub struct Assert {
    pub message: Expression,
    pub assertion: Expression,
}

impl Assert {
    pub(super) fn convert(analyzer: &mut Analyzer, ast: syntax::AssertStatement) -> Self {
        let message = ast
            .message
            .map(|ast| Expression::convert(analyzer, ast))
            .unwrap_or_else(|| todo!("pretty print the expression?"));
        let assertion = Expression::convert(analyzer, ast.assertion);
        Self { message, assertion }
    }
}
