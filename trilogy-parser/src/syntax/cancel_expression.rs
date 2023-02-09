use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct CancelExpression {
    start: Token,
    pub expression: Expression,
}
