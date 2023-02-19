use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct CancelStatement {
    start: Token,
    pub expression: Option<Expression>,
}

impl CancelStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(TokenType::KwCancel)
            .expect("Caller should have found this");
        let expression = if parser.check(Expression::PREFIX).is_ok() && !parser.is_line_start {
            Some(Expression::parse(parser)?)
        } else {
            None
        };
        Ok(Self { start, expression })
    }
}

impl Spanned for CancelStatement {
    fn span(&self) -> Span {
        match &self.expression {
            None => self.start.span,
            Some(expression) => self.start.span.union(expression.span()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(cancelstmt_empty: "cancel" => CancelStatement::parse => "(CancelStatement ())");
    test_parse!(cancelstmt_value: "cancel unit" => CancelStatement::parse => "(CancelStatement (Expression::Unit _))");
    test_parse_error!(cancelstmt_invalid_expr: "cancel {}" => CancelStatement::parse);
    test_parse_error!(cancelstmt_line_break: "cancel\nunit" => CancelStatement::parse);
}
