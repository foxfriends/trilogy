use super::*;
use crate::Converter;
use trilogy_parser::{Spanned, syntax};

#[derive(Clone, Debug)]
pub struct Defer {
    pub body: Expression,
}

impl Defer {
    pub(super) fn convert(converter: &mut Converter, ast: syntax::DeferStatement) -> Expression {
        let span = ast.body.span();
        let body = Expression::convert_block(converter, ast.body);
        Expression::defer(span, Self { body })
    }
}
