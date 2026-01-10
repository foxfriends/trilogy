use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug)]
pub struct UsingStatement {
    pub head: Option<DoHead>,
    pub using: Token,
    pub expression: Expression,
    pub span: Span,
}

impl Spanned for UsingStatement {
    fn span(&self) -> Span {
        self.span
    }
}

impl UsingStatement {
    pub(crate) fn parse(parser: &mut Parser, head: Option<DoHead>) -> SyntaxResult<Self> {
        let using = parser.expect(TokenType::KwUsing).unwrap();
        let expression = Expression::parse(parser)?;
        let mut span = using.span.union(expression.span());
        if let Some(head) = &head {
            span = span.union(head.span());
        }
        Ok(Self {
            span,
            head,
            using,
            expression,
        })
    }
}
