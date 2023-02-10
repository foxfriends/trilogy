use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ExitStatement {
    start: Token,
    pub expression: Expression,
}
