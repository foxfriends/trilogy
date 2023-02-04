use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct MatchExpression {
    start: Token,
    pub expression: Expression,
    pub cases: Vec<MatchExpressionCase>,
}

#[derive(Clone, Debug)]
pub struct MatchExpressionCase {
    start: Token,
    pub pattern: Option<Pattern>,
    pub guard: Option<Expression>,
    pub body: Expression,
}
