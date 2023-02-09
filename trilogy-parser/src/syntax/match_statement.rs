use super::*;
use crate::Spanned;
use source_span::Span;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct MatchStatement {
    start: Token,
    pub expression: Expression,
    pub cases: Vec<MatchStatementCase>,
}

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct MatchStatementCase {
    start: Token,
    pub pattern: Option<Pattern>,
    pub guard: Option<Expression>,
    pub body: Block,
}

impl Spanned for MatchStatementCase {
    fn span(&self) -> Span {
        self.start.span.union(self.body.span())
    }
}
