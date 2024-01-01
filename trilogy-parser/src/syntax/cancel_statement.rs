use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

/// A cancel statement.
///
/// ```trilogy
/// cancel unit
/// ```
#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct CancelStatement {
    pub cancel: Token,
    pub expression: Option<Expression>,
}

impl CancelStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let cancel = parser
            .expect(TokenType::KwCancel)
            .expect("Caller should have found this");
        let expression = if parser.check(Expression::PREFIX).is_ok() && !parser.is_line_start {
            Some(Expression::parse(parser)?)
        } else {
            None
        };
        Ok(Self { cancel, expression })
    }
}

impl Spanned for CancelStatement {
    fn span(&self) -> Span {
        match &self.expression {
            None => self.cancel.span,
            Some(expression) => self.cancel.span.union(expression.span()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(cancelstmt_empty: "cancel" => CancelStatement::parse => "(CancelStatement _ ())");
    test_parse!(cancelstmt_value: "cancel unit" => CancelStatement::parse => "(CancelStatement _ (Expression::Unit _))");
    test_parse_error!(cancelstmt_invalid_expr: "cancel {}" => CancelStatement::parse);
    test_parse_error!(cancelstmt_line_break: "cancel\nunit" => CancelStatement::parse);
}
