use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct AssertStatement {
    start: Token,
    pub message: Option<Expression>,
    pub assertion: Expression,
}

impl AssertStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(TokenType::KwAssert)
            .expect("Caller should have found this");
        let mut message = Some(Expression::parse(parser)?);
        let assertion = parser
            .expect(TokenType::KwAs)
            .ok()
            .map(|_| Expression::parse(parser))
            .transpose()?
            .unwrap_or_else(|| message.take().unwrap());
        Ok(Self {
            start,
            message,
            assertion,
        })
    }
}

impl Spanned for AssertStatement {
    fn span(&self) -> Span {
        self.start.span.union(self.assertion.span())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(assert_true: "assert true" => AssertStatement::parse => "(AssertStatement () (Expression::Boolean _))");
    test_parse!(assert_expression: "assert if x then false else true" => AssertStatement::parse => "(AssertStatement () (Expression::IfElse _))");
    test_parse!(assert_with_message: "assert \"message\" as true" => AssertStatement::parse => "(AssertStatement (Expression::String _) (Expression::Boolean _))");
    test_parse!(assert_commas: "assert a, b, c" => AssertStatement::parse => "(AssertStatement () (Expression::Binary _))");
    test_parse_error!(assert_without_expr: "assert" => AssertStatement::parse);
    test_parse_error!(assert_invalid_expr: "assert + 5" => AssertStatement::parse);
}
