use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug)]
pub struct StructPattern {
    pub atom: AtomLiteral,
    pub open_paren: Token,
    pub pattern: Pattern,
    pub close_paren: Token,
    pub span: Span,
}

impl Spanned for StructPattern {
    fn span(&self) -> Span {
        self.span
    }
}

impl StructPattern {
    pub(crate) fn parse(parser: &mut Parser, atom: AtomLiteral) -> SyntaxResult<Self> {
        let open_paren = parser
            .expect(OParen)
            .map_err(|token| parser.expected(token, "expected `(`"))?;
        let pattern = Pattern::parse(parser)?;
        let close_paren = parser
            .expect(CParen)
            .map_err(|token| parser.expected(token, "expected `)`"))?;
        Ok(Self {
            span: atom.span().union(close_paren.span),
            atom,
            open_paren,
            pattern,
            close_paren,
        })
    }
}

impl TryFrom<StructLiteral> for StructPattern {
    type Error = SyntaxError;

    fn try_from(value: StructLiteral) -> Result<Self, Self::Error> {
        Ok(Self {
            span: value.atom.span().union(value.close_paren.span),
            atom: value.atom,
            open_paren: value.open_paren,
            pattern: value.value.try_into()?,
            close_paren: value.close_paren,
        })
    }
}
