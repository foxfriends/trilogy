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
        let was_allowed = converter.scope.allow_break_continue();
        converter.scope.set_allow_break_continue(false);
        let condition = Expression::convert(converter, ast.condition);
        converter.scope.set_allow_break_continue(true);
        let body = Expression::convert_block(converter, ast.body);
        converter.scope.set_allow_break_continue(was_allowed);
        Expression::r#while(span, Self { condition, body })
    }
}
