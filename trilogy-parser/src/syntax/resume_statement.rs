use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct ResumeStatement {
    start: Token,
    pub expression: Option<Expression>,
}
