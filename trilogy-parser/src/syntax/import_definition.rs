use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ImportDefinition {
    start: Token,
    pub names: Vec<Identifier>,
    pub module: ModulePath,
}

impl ImportDefinition {
    pub(crate) fn parse(parser: &mut Parser, start: Token) -> SyntaxResult<Self> {
        let mut names = vec![];
        loop {
            if parser.check(TokenType::KwFrom).is_ok() {
                break;
            }
            names.push(Identifier::parse(parser)?);
            if parser.expect(TokenType::OpComma).is_ok() {
                continue;
            }
            break;
        }
        parser
            .expect(TokenType::KwFrom)
            .map_err(|token| parser.expected(token, "expected keyword `from`"))?;
        let module = ModulePath::parse(parser)?;
        Ok(Self {
            start,
            names,
            module,
        })
    }
}
