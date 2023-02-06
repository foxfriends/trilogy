use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug)]
pub struct ImportDefinition {
    start: Token,
    pub names: Vec<Identifier>,
    pub module: ModulePath,
}

impl ImportDefinition {
    pub(crate) fn parse(
        parser: &mut Parser,
        start: Token,
        first: Identifier,
    ) -> SyntaxResult<Self> {
        let mut names = vec![first];
        while parser.expect(TokenType::OpComma).is_ok() {
            if parser.check(TokenType::KwFrom).is_some() {
                break;
            }
            names.push(Identifier::parse(parser)?);
        }
        parser
            .expect(TokenType::KwFrom)
            .map_err(|token| parser.expected(token, "expected keyword `from`"))?;
        let module = ModulePath::parse(parser)?;
        Ok(Self {
            start,
            names,
            module,
        })
    }
}
