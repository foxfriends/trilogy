use super::*;
use crate::{spanned::Spanned, Parser};
use source_span::Span;
use trilogy_scanner::TokenType;

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct Path {
    pub module: Option<ModulePath>,
    pub member: Identifier,
}

impl Path {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let module = parser
            .check(TokenType::OpAt)
            .ok()
            .map(|_| ())
            .map(|_| ModulePath::parse(parser))
            .transpose()?;
        if module.is_some() {
            parser
                .expect(TokenType::OpColonColon)
                .expect("a path must end with an identifier");
        }
        let member = Identifier::parse(parser)?;
        Ok(Self { module, member })
    }
}

impl Spanned for Path {
    fn span(&self) -> Span {
        match &self.module {
            Some(module) => module.span().union(self.member.span()),
            None => self.member.span(),
        }
    }
}

impl From<Identifier> for Path {
    fn from(member: Identifier) -> Self {
        Self {
            module: None,
            member,
        }
    }
}
