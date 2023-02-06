use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned)]
pub struct ArrayLiteral {
    start: Token,
    pub elements: ArrayElement,
    end: Token,
}

#[derive(Clone, Debug, Spanned)]
pub enum ArrayElement {
    Element(Expression),
    Spread(Expression),
}
