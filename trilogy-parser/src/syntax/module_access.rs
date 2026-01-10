use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug)]
pub struct ModuleAccess {
    pub lhs: Expression,
    pub access: Token,
    pub rhs: Identifier,
    pub span: Span,
}

impl Spanned for ModuleAccess {
    fn span(&self) -> Span {
        self.span
    }
}

impl ModuleAccess {
    pub(crate) fn parse(parser: &mut Parser, lhs: Expression) -> SyntaxResult<Self> {
        let access = parser.expect(TokenType::OpColonColon).unwrap();
        let rhs = Identifier::parse(parser)?;
        Ok(Self {
            span: lhs.span().union(rhs.span()),
            lhs,
            access,
            rhs,
        })
    }
}
