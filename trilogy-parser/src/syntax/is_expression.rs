use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct IsExpression {
    start: Token,
    pub query: Query,
}
