use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ModuleReference {
    pub name: Identifier,
    pub arguments: Option<ModuleArguments>,
}

impl ModuleReference {
    pub(super) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let name = Identifier::parse(parser)?;
        let arguments = ModuleArguments::parse(parser)?;
        Ok(Self { name, arguments })
    }
}

impl Spanned for ModuleReference {
    fn span(&self) -> Span {
        match &self.arguments {
            Some(arguments) => self.name.span().union(arguments.span()),
            None => self.name.span(),
        }
    }
}

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ModuleArguments {
    start: Token,
    pub arguments: Vec<ModuleReference>,
    end: Token,
}

impl ModuleArguments {
    fn parse(parser: &mut Parser) -> SyntaxResult<Option<Self>> {
        let Ok(start) = parser.expect(TokenType::OParen) else {
            return Ok(None);
        };
        let mut arguments = vec![];
        loop {
            if let Ok(end) = parser.expect(TokenType::CParen) {
                return Ok(Some(Self {
                    start,
                    arguments,
                    end,
                }));
            }
            arguments.push(ModuleReference::parse(parser)?);
            if parser.expect(TokenType::OpComma).is_ok() {
                continue;
            }
            if let Ok(end) = parser.expect(TokenType::CParen) {
                return Ok(Some(Self {
                    start,
                    arguments,
                    end,
                }));
            }
        }
    }
}

impl Spanned for ModuleArguments {
    fn span(&self) -> Span {
        self.start.span.union(self.end.span)
    }
}
