use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned)]
pub struct LetExpression {
    start: Token,
    pub unification: Query,
    pub body: Expression,
}
