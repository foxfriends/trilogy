use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct WhenHandler {
    start: Token,
    pub pattern: Pattern,
    pub strategy: HandlerStrategy,
    pub body: HandlerBody,
}

#[derive(Clone, Debug)]
pub enum HandlerStrategy {
    Cancel(Token),
    Resume(Token),
    Invert(Token),
}

#[derive(Clone, Debug)]
pub enum HandlerBody {
    Block(Box<Block>),
    Expression(Box<Expression>),
}
