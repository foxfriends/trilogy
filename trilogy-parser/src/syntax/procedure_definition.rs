use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ProcedureDefinition {
    start: Token,
    pub head: ProcedureHead,
    pub body: Block,
}

impl ProcedureDefinition {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(TokenType::KwProc)
            .expect("Caller should find `proc` keyword.");
        let head = ProcedureHead::parse(parser)?;
        let body = Block::parse(parser)?;
        Ok(Self { start, head, body })
    }
}
