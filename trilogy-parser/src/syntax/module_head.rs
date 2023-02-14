use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ModuleHead {
    start: Token,
    pub name: Identifier,
    pub parameters: Vec<Identifier>,
}

impl ModuleHead {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(TokenType::KwModule)
            .expect("Caller should find `module` keyword.");
        let name = Identifier::parse(parser)?;
        let mut parameters = vec![];
        while parser.check(TokenType::Identifier).is_ok() {
            parameters.push(Identifier::parse(parser)?);
        }
        Ok(Self {
            start,
            name,
            parameters,
        })
    }
}
