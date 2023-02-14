use super::*;
use crate::Parser;
use trilogy_scanner::TokenType;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ModulePath {
    pub modules: Vec<ModuleReference>,
}

impl ModulePath {
    pub(crate) fn parse_rest(parser: &mut Parser, first: ModuleReference) -> SyntaxResult<Self> {
        let mut modules = vec![first];
        loop {
            if parser.expect(TokenType::OpColonColon).is_err() {
                break;
            }
            modules.push(ModuleReference::parse(parser)?);
        }
        Ok(Self { modules })
    }

    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let first = ModuleReference::parse(parser)?;
        Self::parse_rest(parser, first)
    }

    pub(crate) fn parse_or_path(parser: &mut Parser) -> SyntaxResult<Result<Self, Path>> {
        let mut modules = vec![ModuleReference::parse(parser)?];
        loop {
            if parser.expect(TokenType::OpColonColon).is_err() {
                break;
            }
            if parser.check(TokenType::OpAt).is_ok() {
                modules.push(ModuleReference::parse(parser)?);
            } else {
                let module = Self { modules };
                let member = Identifier::parse(parser)?;
                return Ok(Err(Path {
                    module: Some(module),
                    member,
                }));
            }
        }
        Ok(Ok(Self { modules }))
    }
}
