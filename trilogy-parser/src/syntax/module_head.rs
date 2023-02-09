use super::*;
use crate::{Parser, Spanned};
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ModuleHead {
    start: Token,
    pub name: Identifier,
    pub parameters: Option<ModuleParameters>,
}

impl ModuleHead {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(TokenType::KwModule)
            .expect("Caller should find `module` keyword.");
        let name = Identifier::parse(parser)?;
        let parameters = ModuleParameters::parse(parser)?;
        Ok(Self {
            start,
            name,
            parameters,
        })
    }
}

impl Spanned for ModuleHead {
    fn span(&self) -> source_span::Span {
        match &self.parameters {
            None => self.start.span.union(self.name.span()),
            Some(parameters) => self.start.span.union(parameters.span()),
        }
    }
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ModuleParameters {
    start: Token,
    pub parameters: Vec<Identifier>,
    end: Token,
}

impl ModuleParameters {
    fn parse(parser: &mut Parser) -> SyntaxResult<Option<Self>> {
        let Ok(start) = parser.expect(TokenType::OParen) else {
            return Ok(None);
        };
        let mut parameters = vec![];
        loop {
            if let Ok(end) = parser.expect(TokenType::CParen) {
                return Ok(Some(Self {
                    start,
                    parameters,
                    end,
                }));
            }
            parameters.push(Identifier::parse(parser)?);
            if parser.expect(TokenType::OpComma).is_ok() {
                continue;
            }
            if let Ok(end) = parser.expect(TokenType::CParen) {
                return Ok(Some(Self {
                    start,
                    parameters,
                    end,
                }));
            }
        }
    }
}
