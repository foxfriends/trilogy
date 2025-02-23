use super::*;
use crate::Converter;
use trilogy_parser::{Spanned, syntax};

#[derive(Clone, Debug)]
pub struct While {
    pub condition: Expression,
    pub body: Expression,
}

impl While {
    pub(super) fn convert(converter: &mut Converter, ast: syntax::WhileStatement) -> Expression {
        let span = ast.span();
        let condition = Expression::convert(converter, ast.condition);
        let body = Expression::convert_block(converter, ast.body);
        Expression::r#while(span, Self { condition, body })
    }
}
