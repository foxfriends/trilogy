use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct ResumeExpression {
    start: Token,
    pub expression: Expression,
}
