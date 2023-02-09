use super::*;
use crate::Spanned;
use source_span::Span;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct MatchExpression {
    start: Token,
    pub expression: Expression,
    pub cases: Vec<MatchExpressionCase>,
}

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct MatchExpressionCase {
    start: Token,
    pub pattern: Option<Pattern>,
    pub guard: Option<Expression>,
    pub body: Expression,
}

impl Spanned for MatchExpressionCase {
    fn span(&self) -> Span {
        self.start.span.union(self.body.span())
    }
}
