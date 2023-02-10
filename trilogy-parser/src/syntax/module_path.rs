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

    pub(crate) fn parse_extend(mut self, parser: &mut Parser) -> SyntaxResult<Self> {
        self.modules.push(ModuleReference::parse(parser)?);
        Ok(self)
    }

    pub(crate) fn parse_arguments(mut self, parser: &mut Parser) -> SyntaxResult<Self> {
        let last = self.modules.pop().unwrap();
        self.modules.push(last.parse_arguments(parser)?);
        Ok(self)
    }
}

impl From<Identifier> for ModulePath {
    fn from(value: Identifier) -> Self {
        Self {
            modules: vec![value.into()],
        }
    }
}
