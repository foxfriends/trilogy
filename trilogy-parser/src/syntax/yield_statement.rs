use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct YieldStatement {
    start: Token,
    pub expression: Expression,
}
