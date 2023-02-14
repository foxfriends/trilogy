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
        let end = loop {
            if let Ok(end) = parser.expect(TokenType::CParen) {
                break end;
            }
            parameters.push(Pattern::parse(parser)?);
            if parser.expect(TokenType::OpComma).is_ok() {
                continue;
            }
            break parser.expect(TokenType::CParen).map_err(|token| {
                parser.expected(token, "expected `,` or `)` in parameter list")
            })?;
        };
        Ok(Self {
            name,
            parameters,
            end,
        })
    }
}
