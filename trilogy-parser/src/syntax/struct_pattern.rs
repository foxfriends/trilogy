use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned)]
pub struct StructPattern {
    pub atom: AtomLiteral,
    pub pattern: ValuePattern,
    end: Token,
}
