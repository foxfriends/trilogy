use super::*;
use crate::{Parser, Spanned};
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ModulePath {
    pub first: ModuleReference,
    pub modules: Vec<(Token, ModuleReference)>,
}

impl Spanned for ModulePath {
    fn span(&self) -> source_span::Span {
        if self.modules.is_empty() {
            self.first.span()
        } else {
            self.first
                .span()
                .union(self.modules.last().unwrap().1.span())
        }
    }
}

impl ModulePath {
    pub(crate) fn parse_rest(parser: &mut Parser, first: ModuleReference) -> SyntaxResult<Self> {
        let mut modules = vec![];
        loop {
            let Ok(token) = parser.expect(TokenType::OpColonColon) else {
                break;
            };
            modules.push((token, ModuleReference::parse(parser)?));
        }
        Ok(Self { first, modules })
    }

    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let first = ModuleReference::parse(parser)?;
        Self::parse_rest(parser, first)
    }

    pub(crate) fn parse_or_path(parser: &mut Parser) -> SyntaxResult<Result<Self, Path>> {
        let first = ModuleReference::parse(parser)?;
        let mut modules = vec![];
        loop {
            let Ok(token) = parser.expect(TokenType::OpColonColon) else {
                break;
            };
            if parser.check(TokenType::OpAt).is_ok() {
                modules.push((token, ModuleReference::parse(parser)?));
            } else {
                let module = Self { first, modules };
                let member = Identifier::parse(parser)?;
                return Ok(Err(Path {
                    module: Some(module),
                    join_token: Some(token),
                    member,
                }));
            }
        }
        Ok(Ok(Self { first, modules }))
    }
}
