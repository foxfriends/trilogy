use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct StructPattern {
    pub atom: AtomLiteral,
    pub pattern: Pattern,
    end: Token,
}
