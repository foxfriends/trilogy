use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct StructLiteral {
    pub name: AtomLiteral,
    pub value: Expression,
    end: Token,
}
