use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct UnaryOperation {
    pub operator: UnaryOperator,
    pub operand: Expression,
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum UnaryOperator {
    Negate(Token),
    Not(Token),
    Invert(Token),
    Yield(Token),
}
