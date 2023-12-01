use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ModuleUse {
    pub kw_use: Token,
    pub names: Vec<Identifier>,
}

impl ModuleUse {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let kw_use = parser
            .expect(TokenType::KwUse)
            .expect("caller should find `use` keyword");

        let mut names = vec![];
        while {
            names.push(Identifier::parse(parser)?);
            parser.expect(TokenType::OpComma).is_ok()
        } {}

        Ok(Self { kw_use, names })
    }
}
