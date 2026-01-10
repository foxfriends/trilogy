use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug)]
pub struct DoHead {
    pub r#do: Token,
    pub parameter_list: Option<ParameterList>,
    pub span: Span,
}

impl DoHead {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#do = parser.expect(TokenType::KwDo).unwrap();
        let parameter_list = if let Ok(token) = parser.expect(TokenType::OParen) {
            Some(ParameterList::parse_opened(parser, token)?)
        } else if let Ok((a, b)) = parser.expect_bang_oparen() {
            parser.error(ErrorKind::DoUnnecessaryBangOParen.at(a.span.union(b.span)));
            Some(ParameterList::parse_opened(parser, b)?)
        } else {
            None
        };
        Ok(Self {
            span: match &parameter_list {
                None => r#do.span,
                Some(pl) => r#do.span.union(pl.span),
            },
            r#do,
            parameter_list,
        })
    }
}

impl Spanned for DoHead {
    fn span(&self) -> Span {
        self.span
    }
}
