use super::{expression::Precedence, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

/// A function closure `fn` expression.
///
/// ```trilogy
/// fn x y. x + y
/// ```
#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct FnExpression {
    pub r#fn: Token,
    pub parameters: Vec<Pattern>,
    pub dot: Token,
    pub body: Expression,
    span: Span,
}

impl Spanned for FnExpression {
    fn span(&self) -> Span {
        self.span
    }
}

impl FnExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#fn = parser.expect(KwFn).expect("Caller should have found this");
        let mut parameters = vec![];
        let dot = loop {
            parameters.push(Pattern::parse(parser)?);
            if let Ok(dot) = parser.expect(OpDot) {
                break dot;
            }
        };
        let body = Expression::parse_precedence(parser, Precedence::Continuation)?;
        Ok(Self {
            span: r#fn.span.union(body.span()),
            r#fn,
            parameters,
            dot,
            body,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(fn_identity: "fn x. x" => Expression::parse => "(Expression::Fn (FnExpression _ [(Pattern::Binding _)] _ _))");
    test_parse!(fn_pattern: "fn x:xs. xs" => Expression::parse => "(Expression::Fn (FnExpression _ [(Pattern::Tuple _)] _ _))");
    test_parse!(fn_multiple_params: "fn x y. x * y" => Expression::parse => "(Expression::Fn (FnExpression _ [(Pattern::Binding _) (Pattern::Binding _)] _ _))");
    test_parse!(fn_multiple_patterns: "fn x:xs y:ys. x * y" => Expression::parse => "(Expression::Fn (FnExpression _ [(Pattern::Tuple _) (Pattern::Tuple _)] _ _))");
    test_parse_error!(fn_invalid_body: "fn x. { return x * y }" => Expression::parse);
    test_parse_error!(fn_invalid_pattern: "fn x + y. x" => Expression::parse);
    test_parse_error!(fn_no_params: "fn. 3" => Expression::parse);
}
