use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct GivenHandler {
    start: Token,
    pub head: RuleHead,
    pub body: Option<Query>,
}

impl GivenHandler {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(KwGiven)
            .expect("Caller should have found this");
        let head = RuleHead::parse(parser)?;
        if parser.expect(OpLeftArrow).is_ok() {
            let body = Query::parse(parser)?;
            Ok(Self {
                start,
                head,
                body: Some(body),
            })
        } else {
            Ok(Self {
                start,
                head,
                body: None,
            })
        }
    }
}

impl Spanned for GivenHandler {
    fn span(&self) -> Span {
        match &self.body {
            None => self.start.span.union(self.head.span()),
            Some(body) => self.start.span.union(body.span()),
        }
    }
}
