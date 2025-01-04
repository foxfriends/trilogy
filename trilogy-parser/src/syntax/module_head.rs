use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ModuleHead {
    pub module: Token,
    pub name: Identifier,
    pub parameters: Vec<Identifier>,
    span: Span,
}

impl Spanned for ModuleHead {
    fn span(&self) -> Span {
        self.span
    }
}

impl ModuleHead {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let module = parser.expect(TokenType::KwModule).unwrap();
        let name = Identifier::parse(parser)?;
        let mut parameters = vec![];
        while parser.check(TokenType::Identifier).is_ok() {
            parameters.push(Identifier::parse(parser)?);
        }

        let span = match parameters.last() {
            Some(param) => module.span.union(param.span()),
            None => module.span.union(name.span()),
        };

        Ok(Self {
            span,
            module,
            name,
            parameters,
        })
    }
}
