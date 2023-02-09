use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct IteratorComprehension {
    start: Token,
    pub expression: Expression,
    pub query: Query,
    end: Token,
}
