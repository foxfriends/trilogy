use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned)]
pub struct RecordLiteral {
    start: Token,
    pub elements: RecordElement,
    end: Token,
}

#[derive(Clone, Debug, Spanned)]
pub enum RecordElement {
    Element(Expression, Expression),
    Spread(Expression),
}
