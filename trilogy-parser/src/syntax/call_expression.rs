use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

/// A procedure call expression.
///
/// ```trilogy
/// procedure!(args)
/// ```
#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct CallExpression {
    pub procedure: Expression,
    pub bang: Token,
    pub oparen: Token,
    pub arguments: Punctuated<Expression>,
    pub cparen: Token,
}

impl Spanned for CallExpression {
    fn span(&self) -> Span {
        self.procedure.span().union(self.cparen.span)
    }
}

impl CallExpression {
    pub(crate) fn parse(parser: &mut Parser, procedure: Expression) -> SyntaxResult<Self> {
        let (bang, oparen) = parser.expect_bang_oparen().map_err(|token| {
            parser.expected(
                token,
                "expected `!(` in procedure call, there may not be a space",
            )
        })?;
        let mut arguments = Punctuated::new();
        let cparen = loop {
            if let Ok(end) = parser.expect(CParen) {
                break end;
            }
            let expression = Expression::parse_parameter_list(parser)?.map_err(|patt| {
                let error = SyntaxError::new(
                    patt.span(),
                    "expected an expression in procedure call arguments, but found a pattern",
                );
                parser.error(error.clone());
                error
            })?;
            if let Ok(comma) = parser.expect(OpComma) {
                arguments.push(expression, comma);
                continue;
            }
            arguments.push_last(expression);
            break parser
                .expect(CParen)
                .map_err(|token| parser.expected(token, "expected `,` or `)` in argument list"))?;
        };
        Ok(Self {
            procedure,
            bang,
            oparen,
            arguments,
            cparen,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(callexpr_empty: "hello!()" => Expression::parse => "(Expression::Call (CallExpression _ _ _ [] _))");
    test_parse!(callexpr_params: "hello!(a, b)" => Expression::parse => "(Expression::Call (CallExpression _ _ _ [_ _] _))");
    test_parse!(callexpr_path: "hello world::inner::hello!(2)" => Expression::parse => "(Expression::Call (CallExpression _ _ _ [_] _))");
    test_parse!(callexpr_expr: "(a |> b |> c)!(1)" => Expression::parse => "(Expression::Call (CallExpression _ _ _ [_] _))");
    test_parse_error!(callexpr_spaced: "hello! (1)" => Expression::parse => "expected `!(` in procedure call, there may not be a space");
    test_parse_error!(callexpr_missing_end: "hello!(1" => Expression::parse => "expected `,` or `)` in argument list");
}
