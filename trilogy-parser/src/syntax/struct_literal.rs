use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug)]
pub struct StructLiteral {
    pub atom: AtomLiteral,
    pub open_paren: Token,
    pub value: Expression,
    pub close_paren: Token,
    pub span: Span,
}

impl Spanned for StructLiteral {
    fn span(&self) -> source_span::Span {
        self.span
    }
}

impl StructLiteral {
    pub(crate) fn parse(
        parser: &mut Parser,
        atom: AtomLiteral,
    ) -> SyntaxResult<Result<Self, StructPattern>> {
        let open_paren = parser
            .expect(OParen)
            .map_err(|token| parser.expected(token, "expected `(`"))?;
        let value = Expression::parse_or_pattern(parser)?;
        let close_paren = parser
            .expect(CParen)
            .map_err(|token| parser.expected(token, "expected `)`"))?;

        match value {
            Ok(value) => Ok(Ok(Self {
                span: atom.span().union(close_paren.span),
                atom,
                open_paren,
                value,
                close_paren,
            })),
            Err(pattern) => Ok(Err(StructPattern {
                span: atom.span().union(close_paren.span),
                atom,
                open_paren,
                pattern,
                close_paren,
            })),
        }
    }
}
