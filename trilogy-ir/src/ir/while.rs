use super::*;
use crate::Analyzer;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct While {
    pub condition: Expression,
    pub body: Expression,
}

impl While {
    pub(super) fn convert(analyzer: &mut Analyzer, ast: syntax::WhileStatement) -> Expression {
        let span = ast.span();
        let condition = Expression::convert(analyzer, ast.condition);
        let body = Expression::convert_block(analyzer, ast.body);
        Expression::r#while(span, Self { condition, body })
    }
}
