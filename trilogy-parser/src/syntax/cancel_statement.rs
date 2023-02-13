use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct CancelStatement {
    start: Token,
    pub expression: Option<Expression>,
}

impl CancelStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(TokenType::KwCancel)
            .expect("Caller should have found this");
        if parser.check(TokenType::OpSemi).is_ok() || parser.is_line_start {
            return Ok(Self {
                start,
                expression: None,
            });
        }
        let expression = Expression::parse(parser)?;
        Ok(Self {
            start,
            expression: Some(expression),
        })
    }
}

impl Spanned for CancelStatement {
    fn span(&self) -> Span {
        match &self.expression {
            None => self.start.span,
            Some(expression) => self.start.span.union(expression.span()),
        }
    }
}
