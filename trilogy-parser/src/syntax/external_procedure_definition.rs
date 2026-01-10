use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug)]
pub struct ExternalProcedureDefinition {
    pub r#extern: Token,
    pub call_conv: Token,
    pub proc: Token,
    pub head: ProcedureHead,
    pub span: Span,
}

impl Spanned for ExternalProcedureDefinition {
    fn span(&self) -> Span {
        self.span
    }
}

impl ExternalProcedureDefinition {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#extern = parser.expect(TokenType::KwExtern).unwrap();
        let call_conv = parser.expect(TokenType::String).unwrap();
        let proc = parser.expect(TokenType::KwProc).map_err(|token| {
            parser.expected(token, "expected proc keyword to begin extern definition")
        })?;

        let head = ProcedureHead::parse(parser)?;
        Ok(Self {
            span: r#extern.span.union(head.span()),
            r#extern,
            call_conv,
            proc,
            head,
        })
    }
}
