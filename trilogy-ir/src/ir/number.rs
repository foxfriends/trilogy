use num::{BigRational, Complex};
use trilogy_parser::syntax;

#[derive(Clone, Debug)]
pub struct Number(Complex<BigRational>);

impl Number {
    pub(super) fn convert(ast: syntax::NumberLiteral) -> Self {
        Self(ast.value())
    }
}
