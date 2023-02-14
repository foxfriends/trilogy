use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct Lookup {
    pub path: Path,
    pub patterns: Vec<Pattern>,
    end: Token,
}

impl Lookup {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let path = Path::parse(parser)?;
        parser
            .expect(TokenType::OParen)
            .map_err(|token| parser.expected(token, "expected `(`"))?;
        let mut patterns = vec![];
        let end = loop {
            if let Ok(end) = parser.expect(TokenType::CParen) {
                break end;
            }
            patterns.push(Pattern::parse(parser)?);
            if parser.expect(TokenType::OpComma).is_ok() {
                continue;
            }
            break parser.expect(TokenType::CParen).map_err(|token| {
                parser.expected(token, "expected `,` or `)` in parameter list")
            })?;
        };
        Ok(Self {
            path,
            patterns,
            end,
        })
    }
}
