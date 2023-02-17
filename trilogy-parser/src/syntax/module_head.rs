use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ModuleHead {
    start: Token,
    pub name: Identifier,
    pub parameters: Vec<Identifier>,
}

impl Spanned for ModuleHead {
    fn span(&self) -> Span {
        self.start.span.union(if self.parameters.is_empty() {
            self.name.span()
        } else {
            self.parameters.span()
        })
    }
}

impl ModuleHead {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(TokenType::KwModule)
            .expect("Caller should find `module` keyword.");
        let name = Identifier::parse(parser)?;
        let mut parameters = vec![];
        while parser.check(TokenType::Identifier).is_ok() {
            parameters.push(Identifier::parse(parser)?);
        }
        Ok(Self {
            start,
            name,
            parameters,
        })
    }
}
