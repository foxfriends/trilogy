use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug)]
pub struct WhileStatement {
    pub r#while: Token,
    pub condition: Expression,
    pub body: Block,
    pub span: Span,
}

impl Spanned for WhileStatement {
    fn span(&self) -> Span {
        self.span
    }
}

impl WhileStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#while = parser.expect(TokenType::KwWhile).unwrap();
        let condition = Expression::parse(parser)?;
        let body = Block::parse(parser)?;
        Ok(Self {
            span: r#while.span.union(body.span()),
            r#while,
            condition,
            body,
        })
    }
}
