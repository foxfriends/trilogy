use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct ArrayLiteral {
    start: Token,
    pub elements: ArrayElement,
    end: Token,
}

#[derive(Clone, Debug)]
pub enum ArrayElement {
    Element(Expression),
    Spread(Expression),
}
