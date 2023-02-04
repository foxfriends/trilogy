use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct LetExpression {
    start: Token,
    pub unification: Unification,
    pub body: Expression,
}
