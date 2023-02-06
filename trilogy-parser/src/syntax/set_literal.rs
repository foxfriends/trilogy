use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned)]
pub struct SetLiteral {
    start: Token,
    pub elements: SetElement,
    end: Token,
}

#[derive(Clone, Debug, Spanned)]
pub enum SetElement {
    Element(Expression),
    Spread(Expression),
}
