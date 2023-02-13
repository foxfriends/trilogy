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
        procedure: Expression,
    ) -> SyntaxResult<Self> {
        parser
            .expect_bang_oparen()
            .map_err(|token| parser.expected(token, "expected `!(` in procedure call"))?;
        let mut arguments = vec![];
        loop {
            if parser.check(CParen).is_ok() {
                break;
            }
            arguments.push(Expression::parse_parameter_list(parser)?);
            if parser.expect(OpComma).is_ok() {
                continue;
            }
        }
        let end = parser
            .expect(CParen)
            .map_err(|token| parser.expected(token, "expected `,` or `)` in argument list"))?;
        Ok(Self {
            procedure,
            arguments,
            end,
        })
    }
}
