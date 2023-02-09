use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct NegativePattern {
    start: Token,
    pub pattern: ValuePattern,
}
