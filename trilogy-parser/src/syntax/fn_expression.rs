use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct FnExpression {
    start: Token,
    pub parameters: Vec<Pattern>,
    pub body: Expression,
}
