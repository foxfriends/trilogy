use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct RuleHead {
    pub name: Identifier,
    pub open_paren: Token,
    pub parameters: Vec<Pattern>,
    pub close_paren: Token,
}

impl Spanned for RuleHead {
    fn span(&self) -> Span {
        self.name.span().union(self.close_paren.span)
    }
}

impl RuleHead {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let name = Identifier::parse(parser)?;
        let open_paren = parser
            .expect(TokenType::OParen)
            .map_err(|token| parser.expected(token, "expected `(`"))?;
        let mut parameters = vec![];
        let close_paren = loop {
            if let Ok(close_paren) = parser.expect(TokenType::CParen) {
                break close_paren;
            }
            parameters.push(Pattern::parse(parser)?);
            if parser.expect(TokenType::OpComma).is_ok() {
                continue;
            }
            break parser.expect(TokenType::CParen).map_err(|token| {
                parser.expected(token, "expected `,` or `)` in parameter list")
            })?;
        };
        Ok(Self {
            name,
            open_paren,
            parameters,
            close_paren,
        })
    }
}
