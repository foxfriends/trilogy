use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct WhileStatement {
    start: Token,
    pub condition: Expression,
    pub body: Block,
}
