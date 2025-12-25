use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct Guard {
    pub r#if: Token,
    pub expression: Expression,
    span: Span,
}

impl Spanned for Guard {
    fn span(&self) -> Span {
        self.span
    }
}

impl Guard {
    pub(crate) fn parse_optional(parser: &mut Parser) -> SyntaxResult<Option<Self>> {
        let Ok(r#if) = parser.expect(KwIf) else {
            return Ok(None);
        };
        let expression = Expression::parse(parser)?;
        Ok(Some(Self {
            span: r#if.span.union(expression.span()),
            r#if,
            expression,
        }))
    }
}
