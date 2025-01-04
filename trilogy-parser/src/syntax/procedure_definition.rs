use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ProcedureDefinition {
    pub proc: Token,
    pub head: ProcedureHead,
    pub body: Block,
    span: Span,
}

impl Spanned for ProcedureDefinition {
    fn span(&self) -> Span {
        self.span
    }
}

impl ProcedureDefinition {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let proc = parser.expect(TokenType::KwProc).unwrap();
        let head = ProcedureHead::parse(parser)?;
        let body = Block::parse(parser)?;
        Ok(Self {
            span: proc.span.union(body.span()),
            proc,
            head,
            body,
        })
    }
}
