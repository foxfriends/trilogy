use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug)]
pub struct ModuleHead {
    start: Token,
    pub name: Identifier,
    pub parameters: Option<ModuleParameters>,
}

#[derive(Clone, Debug)]
pub struct ModuleParameters {
    start: Token,
    pub parameters: Vec<Identifier>,
    end: Token,
}

impl ModuleHead {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(TokenType::KwModule)
            .expect("Caller should find `module` keyword.");
        let name = Identifier::parse(parser)?;
        let parameters = ModuleParameters::parse(parser)?;
        Ok(Self {
            start,
            name,
            parameters,
        })
    }
}

impl ModuleParameters {
    fn parse(parser: &mut Parser) -> SyntaxResult<Option<Self>> {
        let Ok(start) = parser.expect(TokenType::OParen) else {
            return Ok(None);
        };
        let mut parameters = vec![];
        loop {
            if let Ok(end) = parser.expect(TokenType::CParen) {
                return Ok(Some(Self {
                    start,
                    parameters,
                    end,
                }));
            }
            parameters.push(Identifier::parse(parser)?);
            if parser.expect(TokenType::OpComma).is_ok() {
                continue;
            }
            if let Ok(end) = parser.expect(TokenType::CParen) {
                return Ok(Some(Self {
                    start,
                    parameters,
                    end,
                }));
            }
        }
    }
}
