use super::{Identifier, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{
    Token,
    TokenType::{self, *},
};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ElseHandler {
    start: Token,
    pub identifier: Option<Identifier>,
    pub strategy: HandlerStrategy,
    pub body: Option<HandlerBody>,
}

impl ElseHandler {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(KwElse)
            .expect("Caller should have found this");

        let identifier = if parser.check(TokenType::Identifier).is_ok() {
            Some(Identifier::parse(parser)?)
        } else {
            None
        };

        let strategy = HandlerStrategy::parse(parser)?;
        let body = if !matches!(strategy, HandlerStrategy::Yield(..)) {
            Some(HandlerBody::parse(parser)?)
        } else {
            None
        };

        Ok(Self {
            start,
            identifier,
            strategy,
            body,
        })
    }
}

impl Spanned for ElseHandler {
    fn span(&self) -> Span {
        match &self.body {
            None => self.start.span.union(self.strategy.span()),
            Some(body) => self.start.span.union(body.span()),
        }
    }
}
