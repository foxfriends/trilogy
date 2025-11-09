use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct TypeHead {
    pub r#type: Token,
    pub name: Identifier,
    pub parameters: Vec<Identifier>,
    span: Span,
}

impl Spanned for TypeHead {
    fn span(&self) -> Span {
        self.span
    }
}

impl TypeHead {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#type = parser.expect(TokenType::KwType).unwrap();
        let name = Identifier::parse(parser)?;
        let mut parameters = vec![];
        while parser.check(TokenType::Identifier).is_ok() {
            parameters.push(Identifier::parse(parser)?);
        }

        let span = match parameters.last() {
            Some(param) => r#type.span.union(param.span()),
            None => r#type.span.union(name.span()),
        };

        Ok(Self {
            span,
            r#type,
            name,
            parameters,
        })
    }
}
