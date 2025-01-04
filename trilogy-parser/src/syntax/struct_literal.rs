use super::*;
use crate::{Parser, Spanned};
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct StructLiteral {
    pub atom: AtomLiteral,
    pub open_paren: Token,
    pub value: Expression,
    pub close_paren: Token,
}

impl Spanned for StructLiteral {
    fn span(&self) -> source_span::Span {
        self.atom.span().union(self.close_paren.span)
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
                atom,
                open_paren,
                value,
                close_paren,
            })),
            Err(pattern) => Ok(Err(StructPattern {
                atom,
                open_paren,
                pattern,
                close_paren,
            })),
        }
    }
}
