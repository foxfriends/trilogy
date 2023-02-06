use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned)]
pub struct WhenHandler {
    start: Token,
    pub pattern: Pattern,
    pub strategy: HandlerStrategy,
    pub body: HandlerBody,
}

#[derive(Clone, Debug, Spanned)]
pub enum HandlerStrategy {
    Cancel(Token),
    Resume(Token),
    Invert(Token),
}

#[derive(Clone, Debug, Spanned)]
pub enum HandlerBody {
    Block(Box<Block>),
    Expression(Box<Expression>),
}
