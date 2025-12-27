use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct DoHead {
    pub r#do: Token,
    pub parameter_list: Option<ParameterList>,
}

impl DoHead {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#do = parser.expect(TokenType::KwDo).unwrap();
        let parameter_list = if parser.check(TokenType::OParen).is_ok() {
            Some(ParameterList::parse(parser)?)
        } else {
            None
        };
        Ok(Self {
            r#do,
            parameter_list,
        })
    }
}

impl Spanned for DoHead {
    fn span(&self) -> Span {
        match &self.parameter_list {
            None => self.r#do.span,
            Some(pl) => self.r#do.span.union(pl.span()),
        }
    }
}
