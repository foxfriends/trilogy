use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ModuleAccess {
    pub lhs: Expression,
    access: Token,
    pub rhs: Identifier,
}

impl ModuleAccess {
    pub(crate) fn parse(parser: &mut Parser, lhs: Expression) -> SyntaxResult<Self> {
        let access = parser.expect(TokenType::OpColonColon).unwrap();
        let rhs = Identifier::parse(parser)?;
        Ok(Self { lhs, access, rhs })
    }

    pub fn access_token(&self) -> &Token {
        &self.access
    }
}
