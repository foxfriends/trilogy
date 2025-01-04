use super::*;
use crate::{Parser, Spanned};
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct StructPattern {
    pub atom: AtomLiteral,
    pub open_paren: Token,
    pub pattern: Pattern,
    pub close_paren: Token,
}

impl Spanned for StructPattern {
    fn span(&self) -> source_span::Span {
        self.atom.span().union(self.close_paren.span)
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
            atom: value.atom,
            open_paren: value.open_paren,
            pattern: value.value.try_into()?,
            close_paren: value.close_paren,
        })
    }
}
