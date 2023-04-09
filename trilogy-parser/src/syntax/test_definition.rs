use super::*;
use crate::{Parser, PrettyPrint, PrettyPrinted, PrettyPrinter};
use pretty::DocAllocator;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct TestDefinition {
    start: Token,
    pub name: StringLiteral,
    pub body: Block,
}

impl TestDefinition {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(TokenType::KwTest)
            .expect("Caller should find `test` keyword.");
        let name = StringLiteral::parse(parser)?;
        let body = Block::parse(parser)?;
        Ok(Self { start, name, body })
    }
}

impl<'a> PrettyPrint<'a> for TestDefinition {
    fn pretty_print(&self, printer: &'a PrettyPrinter) -> PrettyPrinted<'a> {
        printer
            .text("test")
            .append(printer.space())
            .append(self.name.pretty_print(printer))
            .append(printer.space())
            .append(self.body.pretty_print(printer))
    }
}
