use super::*;
use crate::Parser;
use trilogy_scanner::TokenType;

#[derive(Clone, Debug, Spanned)]
pub struct FunctionHead {
    pub name: Identifier,
    pub parameters: Vec<Pattern>,
}

impl FunctionHead {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let name = Identifier::parse(parser)?;
        let mut parameters = vec![];
        loop {
            parameters.push(Pattern::parse(parser)?);
            if parser.expect(TokenType::OpEq).is_ok() {
                return Ok(Self { name, parameters });
            }
        }
    }
}
