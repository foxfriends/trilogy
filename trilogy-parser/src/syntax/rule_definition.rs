use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct RuleDefinition {
    start: Token,
    pub head: RuleHead,
    pub body: Option<Query>,
}

impl RuleDefinition {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(TokenType::KwRule)
            .expect("Caller should find `rule` keyword.");
        let head = RuleHead::parse(parser)?;
        let body = parser
            .expect(TokenType::OpLeftArrow)
            .ok()
            .map(|_| Query::parse(parser))
            .transpose()?;
        Ok(Self { start, head, body })
    }
}

impl Spanned for RuleDefinition {
    fn span(&self) -> Span {
        match &self.body {
            None => self.start.span.union(self.head.span()),
            Some(body) => self.start.span.union(body.span()),
        }
    }
}
