use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct CallExpression {
    pub procedure: Expression,
    start: Token,
    pub arguments: Vec<Expression>,
    end: Token,
}

impl Spanned for CallExpression {
    fn span(&self) -> Span {
        self.procedure.span().union(self.end.span)
    }
}

impl CallExpression {
    pub(crate) fn parse(parser: &mut Parser, procedure: Expression) -> SyntaxResult<Self> {
        let (_, start) = parser.expect_bang_oparen().map_err(|token| {
            parser.expected(
                token,
                "expected `!(` in procedure call, there may not be a space",
            )
        })?;
        let mut arguments = vec![];
        let end = loop {
            if let Ok(end) = parser.expect(CParen) {
                break end;
            }
            arguments.push(Expression::parse_parameter_list(parser)?);
            if parser.expect(OpComma).is_ok() {
                continue;
            }
            break parser
                .expect(CParen)
                .map_err(|token| parser.expected(token, "expected `,` or `)` in argument list"))?;
        };
        Ok(Self {
            procedure,
            start,
            arguments,
            end,
        })
    }

    pub fn start_token(&self) -> &Token {
        &self.start
    }

    pub fn end_token(&self) -> &Token {
        &self.end
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(callexpr_empty: "hello!()" => Expression::parse => "(Expression::Call (CallExpression _ []))");
    test_parse!(callexpr_params: "hello!(a, b)" => Expression::parse => "(Expression::Call (CallExpression _ [_ _]))");
    test_parse!(callexpr_path: "@hello world::@inner::hello!(2)" => Expression::parse => "(Expression::Call (CallExpression _ [_]))");
    test_parse!(callexpr_expr: "(a |> b |> c)!(1)" => Expression::parse => "(Expression::Call (CallExpression _ [_]))");
    test_parse_error!(callexpr_spaced: "hello! (1)" => Expression::parse => "expected `!(` in procedure call, there may not be a space");
    test_parse_error!(callexpr_missing_end: "hello!(1" => Expression::parse => "expected `,` or `)` in argument list");
}
