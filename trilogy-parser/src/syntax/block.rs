use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct Block {
    start: Token,
    pub statements: Vec<Statement>,
    end: Token,
    pub handlers: Vec<Handler>,
}

impl Block {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(OBrace)
            .map_err(|token| parser.expected(token, "expected `{`"))?;

        let end = parser
            .expect(CBrace)
            .map_err(|token| parser.expected(token, "expected `}` to end block"))?;

        Ok(Self {
            start,
            statements: vec![],
            end,
            handlers: vec![],
        })
    }
}
