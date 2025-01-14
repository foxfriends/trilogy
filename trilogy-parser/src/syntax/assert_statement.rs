use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

/// The optional message portion of an assert statement.
///
/// ```trilogy
/// #- assert -# "msg" as #- expression -#
/// ```
#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct AssertMessage {
    pub message: Expression,
    pub r#as: Token,
}

/// An assert statement.
///
/// ```trilogy
/// assert "msg" as expression
/// ```
#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct AssertStatement {
    pub assert: Token,
    pub message: Option<AssertMessage>,
    pub assertion: Expression,
}

impl AssertStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let assert = parser.expect(TokenType::KwAssert).unwrap();
        let message_or_assertion = Expression::parse(parser)?;
        if let Ok(r#as) = parser.expect(TokenType::KwAs) {
            let assertion = Expression::parse(parser)?;
            return Ok(Self {
                assert,
                message: Some(AssertMessage {
                    message: message_or_assertion,
                    r#as,
                }),
                assertion,
            });
        }
        Ok(Self {
            assert,
            message: None,
            assertion: message_or_assertion,
        })
    }
}

impl Spanned for AssertStatement {
    fn span(&self) -> Span {
        self.assert.span.union(self.assertion.span())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(assert_true: "assert true" => AssertStatement::parse => "(AssertStatement _ () (Expression::Boolean _))");
    test_parse!(assert_expression: "assert if x then false else true" => AssertStatement::parse => "(AssertStatement _ () (Expression::IfElse _))");
    test_parse!(assert_with_message: "assert \"message\" as true" => AssertStatement::parse => "(AssertStatement _ (AssertMessage _ _) (Expression::Boolean _))");
    test_parse_error!(assert_without_expr: "assert" => AssertStatement::parse);
    test_parse_error!(assert_invalid_expr: "assert + 5" => AssertStatement::parse);
}
