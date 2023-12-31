use super::*;
use crate::Converter;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Assert {
    pub message: Expression,
    pub assertion: Expression,
}

impl Assert {
    pub(super) fn convert(converter: &mut Converter, ast: syntax::AssertStatement) -> Self {
        let message = ast
            .message
            .map(|ast| Expression::convert(converter, ast.message))
            .unwrap_or_else(|| {
                Expression::string(ast.assertion.span(), "assertion failed".to_owned())
            });
        let assertion = Expression::convert(converter, ast.assertion);
        Self { message, assertion }
    }
}
