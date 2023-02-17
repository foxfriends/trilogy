use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ExportDefinition {
    start: Token,
    pub names: Vec<Identifier>,
}

impl Spanned for ExportDefinition {
    fn span(&self) -> Span {
        if self.names.is_empty() {
            self.start.span
        } else {
            self.start.span.union(self.names.span())
        }
    }
}

impl ExportDefinition {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(TokenType::KwExport)
            .expect("Caller should find `export` keyword.");
        let mut names = vec![];
        while {
            names.push(Identifier::parse(parser)?);
            parser.expect(TokenType::OpComma).is_ok()
        } {}
        Ok(Self { start, names })
    }
}
