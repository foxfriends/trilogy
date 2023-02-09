use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct RuleHead {
    pub name: Identifier,
    pub parameters: Vec<Pattern>,
    end: Token,
}

impl RuleHead {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let name = Identifier::parse(parser)?;
        parser
            .expect(TokenType::OParen)
            .map_err(|token| parser.expected(token, "expected `(`"))?;
        let mut parameters = vec![];
        loop {
            if let Ok(end) = parser.expect(TokenType::CParen) {
                return Ok(Self {
                    name,
                    parameters,
                    end,
                });
            }
            parameters.push(Pattern::parse(parser)?);
            if parser.expect(TokenType::OpComma).is_ok() {
                continue;
            }
            if let Ok(end) = parser.expect(TokenType::CParen) {
                return Ok(Self {
                    name,
                    parameters,
                    end,
                });
            }
        }
    }
}
