use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct SetLiteral {
    start: Token,
    pub elements: SetElement,
    end: Token,
}

#[derive(Clone, Debug)]
pub enum SetElement {
    Element(Expression),
    Spread(Expression),
}
