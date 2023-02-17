use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct Block {
    start: Token,
    pub statements: Vec<Statement>,
    end: Token,
}

impl Spanned for Block {
    fn span(&self) -> Span {
        self.start.span.union(self.end.span)
    }
}

impl Block {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(OBrace)
            .map_err(|token| parser.expected(token, "expected `{`"))?;

        let mut statements = vec![];
        let mut first = true;
        let end = loop {
            if let Ok(end) = parser.expect(CBrace) {
                break end;
            }
            if !first && parser.expect(OpSemi).is_err() && !parser.is_line_start {
                let token = parser.peek();
                let error = SyntaxError::new(
                    token.span,
                    "expected `;` or line break to separate statements",
                );
                parser.error(error);
            }
            statements.push(Statement::parse(parser)?);
            first = false;
        };

        Ok(Self {
            start,
            statements,
            end,
        })
    }
}
