use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct CallExpression {
    pub procedure: Expression,
    pub arguments: Vec<Expression>,
    end: Token,
}

impl CallExpression {
    pub(crate) fn parse(
        parser: &mut Parser,
        procedure: impl Into<Expression>,
    ) -> SyntaxResult<Self> {
        parser
            .expect(BangOParen)
            .expect("Caller should have found this");
        let mut arguments = vec![];
        loop {
            if parser.check(CParen).is_some() {
                break;
            }
            arguments.push(ValueExpression::parse(parser)?.into());
            if parser.expect(OpComma).is_ok() {
                continue;
            }
        }
        let end = parser.expect(CParen).map_err(|token| {
            let error = SyntaxError::new(token.span, "expected `,` or `)` in argument list");
            parser.error(error.clone());
            error
        })?;
        Ok(Self {
            procedure: procedure.into(),
            arguments,
            end,
        })
    }
}
