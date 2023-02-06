use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned)]
pub struct DoExpression {
    start: Token,
    pub parameters: Vec<Pattern>,
    pub body: DoBody,
}

#[derive(Clone, Debug, Spanned)]
pub enum DoBody {
    Block(Box<Block>),
    Expression(Box<Expression>),
}
