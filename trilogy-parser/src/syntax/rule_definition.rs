use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct RuleDefinition {
    pub rule: Token,
    pub head: RuleHead,
    pub body: Option<Query>,
    span: Span,
}

impl RuleDefinition {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let rule = parser.expect(TokenType::KwRule).unwrap();
        let head = RuleHead::parse(parser)?;
        let body = parser
            .expect([TokenType::OpLeftArrow, TokenType::OpRightArrow])
            .inspect(|token| {
                if token.token_type == TokenType::OpRightArrow {
                    // It was an error, but we can almost certainly still parse pretty accurately.
                    parser.error(ErrorKind::RuleRightArrow.at(token.span));
                }
            })
            .ok()
            .map(|_| Query::parse(parser))
            .transpose()?;
        let span = match &body {
            None => rule.span.union(head.span()),
            Some(body) => rule.span.union(body.span()),
        };
        Ok(Self {
            span,
            rule,
            head,
            body,
        })
    }
}

impl Spanned for RuleDefinition {
    fn span(&self) -> Span {
        self.span
    }
}
