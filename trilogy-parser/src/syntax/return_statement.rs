use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ReturnStatement {
    start: Token,
    pub expression: Option<Expression>,
}

impl ReturnStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(TokenType::KwReturn)
            .expect("Caller should have found this");
        let expression = if parser.check(Expression::PREFIX).is_ok() && !parser.is_line_start {
            Some(Expression::parse(parser)?)
        } else {
            None
        };
        Ok(Self { start, expression })
    }

    pub fn return_token(&self) -> &Token {
        &self.start
    }
}

impl Spanned for ReturnStatement {
    fn span(&self) -> Span {
        match &self.expression {
            None => self.start.span,
            Some(expression) => self.start.span.union(expression.span()),
        }
    }
}
