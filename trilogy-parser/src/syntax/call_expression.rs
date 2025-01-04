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
    pub open_paren: Token,
    pub arguments: Punctuated<Expression>,
    pub close_paren: Token,
    span: Span,
}

impl Spanned for CallExpression {
    fn span(&self) -> Span {
        self.span
    }
}

impl CallExpression {
    pub(crate) fn parse(parser: &mut Parser, procedure: Expression) -> SyntaxResult<Self> {
        let (bang, open_paren) = parser.expect_bang_oparen().unwrap();
        let mut arguments = Punctuated::new();
        let close_paren = loop {
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
            span: procedure.span().union(close_paren.span),
            procedure,
            bang,
            open_paren,
            arguments,
            close_paren,
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
    test_parse_error!(callexpr_missing_end: "hello!(1" => Expression::parse => "expected `,` or `)` in argument list");
}
