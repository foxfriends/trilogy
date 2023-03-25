use num::{BigRational, Complex};
use source_span::Span;

#[derive(Clone, Debug)]
pub struct NumberLiteral {
    span: Span,
    pub value: Number,
}

#[derive(Clone, Debug)]
pub struct Number(Complex<BigRational>);
