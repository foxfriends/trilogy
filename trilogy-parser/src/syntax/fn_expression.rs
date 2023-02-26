use super::{expression::Precedence, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct FnExpression {
    start: Token,
    pub parameters: Vec<Pattern>,
    pub body: Expression,
}

impl Spanned for FnExpression {
    fn span(&self) -> Span {
        self.start.span.union(self.body.span())
    }
}

impl FnExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser.expect(KwFn).expect("Caller should have found this");
        let mut parameters = vec![];
        loop {
            parameters.push(Pattern::parse(parser)?);
            if parser.expect(OpDot).is_ok() {
                break;
            }
        }
        let body = Expression::parse_precedence(parser, Precedence::Continuation)?;
        Ok(Self {
            start,
            parameters,
            body,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(fn_identity: "fn x. x" => Expression::parse => "(Expression::Fn (FnExpression [(Pattern::Binding _)] _))");
    test_parse!(fn_pattern: "fn x:xs. xs" => Expression::parse => "(Expression::Fn (FnExpression [(Pattern::Tuple _)] _))");
    test_parse!(fn_multiple_params: "fn x y. x * y" => Expression::parse => "(Expression::Fn (FnExpression [(Pattern::Binding _) (Pattern::Binding _)] _))");
    test_parse!(fn_multiple_patterns: "fn x:xs y:ys. x * y" => Expression::parse => "(Expression::Fn (FnExpression [(Pattern::Tuple _) (Pattern::Tuple _)] _))");
    test_parse_error!(fn_invalid_body: "fn x. { return x * y }" => Expression::parse);
    test_parse_error!(fn_invalid_pattern: "fn x + y. x" => Expression::parse);
    test_parse_error!(fn_no_params: "fn. 3" => Expression::parse);
}
