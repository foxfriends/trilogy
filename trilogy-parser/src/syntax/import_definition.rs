use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ImportDefinition {
    start: Token,
    pub names: Vec<Alias>,
    from: Token,
    pub module: ModulePath,
}

impl Spanned for ImportDefinition {
    fn span(&self) -> Span {
        self.start.span.union(self.module.span())
    }
}

impl ImportDefinition {
    pub(crate) fn parse(parser: &mut Parser, start: Token) -> SyntaxResult<Self> {
        let mut names = vec![];
        loop {
            if parser.check(TokenType::KwFrom).is_ok() {
                break;
            }
            names.push(Alias::parse(parser)?);
            if parser.expect(TokenType::OpComma).is_ok() {
                continue;
            }
            break;
        }
        let from = parser
            .expect(TokenType::KwFrom)
            .map_err(|token| parser.expected(token, "expected keyword `from`"))?;
        let module = ModulePath::parse(parser)?;
        Ok(Self {
            start,
            names,
            from,
            module,
        })
    }

    pub fn from_token(&self) -> &Token {
        &self.from
    }
}
