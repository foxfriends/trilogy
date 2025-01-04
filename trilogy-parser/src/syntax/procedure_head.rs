use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ProcedureHead {
    pub name: Identifier,
    pub bang: Token,
    pub open_paren: Token,
    pub parameters: Vec<Pattern>,
    pub close_paren: Token,
    span: Span,
}

impl Spanned for ProcedureHead {
    fn span(&self) -> Span {
        self.span
    }
}

impl ProcedureHead {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let name = Identifier::parse(parser)?;
        let (bang, open_paren) = parser
            .expect_bang_oparen()
            .map_err(|token| parser.expected(token, "expected `!(`"))?;
        let mut parameters = vec![];
        loop {
            if parser.check(TokenType::CParen).is_ok() {
                break;
            }
            parameters.push(Pattern::parse(parser)?);
            if parser.expect(TokenType::OpComma).is_ok() {
                continue;
            }
        }
        let close_paren = parser
            .expect(TokenType::CParen)
            .map_err(|token| parser.expected(token, "expected `,` or `)` in parameter list"))?;
        Ok(Self {
            span: name.span().union(close_paren.span),
            name,
            bang,
            open_paren,
            parameters,
            close_paren,
        })
    }
}
