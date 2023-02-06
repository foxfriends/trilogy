use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned)]
pub struct CallExpression {
    pub path: Path,
    pub arguments: Vec<Expression>,
    end: Token,
}
