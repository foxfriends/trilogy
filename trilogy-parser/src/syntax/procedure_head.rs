use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ProcedureHead {
    pub name: Identifier,
    pub parameters: Vec<Pattern>,
    end: Token,
}

impl ProcedureHead {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let name = Identifier::parse(parser)?;
        parser
            .expect(TokenType::BangOParen)
            .map_err(|token| parser.expected(token, "expected `!(`"))?;
        let mut parameters = vec![];
        loop {
            if parser.check(TokenType::CParen).is_ok() {
                break;
            }
            parameters.push(Pattern::parse(parser)?);
            if parser.expect(TokenType::OpComma).is_ok() {
                continue;
            }
        }
        let end = parser
            .expect(TokenType::CParen)
            .map_err(|token| parser.expected(token, "expected `,` or `)` in parameter list"))?;
        Ok(Self {
            name,
            parameters,
            end,
        })
    }
}
