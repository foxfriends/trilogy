use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug)]
pub struct ExportDefinition {
    start: Token,
    pub names: Vec<Identifier>,
}

impl ExportDefinition {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(TokenType::KwExport)
            .expect("Caller should find `export` keyword.");
        let mut names = vec![];
        while {
            names.push(Identifier::parse(parser)?);
            parser.expect(TokenType::OpComma).is_ok()
        } {}
        Ok(Self { start, names })
    }
}
