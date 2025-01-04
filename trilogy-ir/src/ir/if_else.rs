use super::*;
use crate::Converter;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct IfElse {
    pub condition: Expression,
    pub when_true: Expression,
    pub when_false: Expression,
}

impl IfElse {
    pub(super) fn new(
        condition: Expression,
        when_true: Expression,
        when_false: Expression,
    ) -> Self {
        Self {
            condition,
            when_true,
            when_false,
        }
    }

    pub(super) fn convert_expression(
        converter: &mut Converter,
        ast: syntax::IfElseExpression,
    ) -> Expression {
        let span = ast.span();
        let condition = Expression::convert(converter, ast.condition);
        let when_false = match ast.when_false {
            Some(when_false) => Expression::convert(converter, when_false.expression),
            None => Expression::unit(span),
        };
        let when_true = match ast.when_true {
            syntax::IfBody::Block(block) => Expression::convert_block(converter, block),
            syntax::IfBody::Then(_, expr) => Expression::convert(converter, expr),
        };
        Expression::if_else(span, Self::new(condition, when_true, when_false))
    }
}
