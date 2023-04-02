use super::*;
use crate::{spanned::Spanned, Parser};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct Path {
    pub module: Option<ModulePath>,
    pub(super) join_token: Option<Token>,
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
        let join_token = if module.is_some() {
            Some(
                parser
                    .expect(TokenType::OpColonColon)
                    .expect("a path must end with an identifier"),
            )
        } else {
            None
        };
        let member = Identifier::parse(parser)?;
        Ok(Self {
            module,
            join_token,
            member,
        })
    }

    pub fn join_token(&self) -> Option<&Token> {
        self.join_token.as_ref()
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
            join_token: None,
            member,
        }
    }
}
