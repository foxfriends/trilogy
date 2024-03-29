use super::{expression::Precedence, *};
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

/// A cancel expression.
///
/// ```trilogy
/// cancel unit
/// ```
#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct CancelExpression {
    pub cancel: Token,
    pub expression: Expression,
}

impl CancelExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let cancel = parser
            .expect(KwCancel)
            .expect("Caller should have found this");
        let expression = Expression::parse_precedence(parser, Precedence::Continuation)?;
        Ok(Self { cancel, expression })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(cancelexpr_unit: "cancel unit" => CancelExpression::parse => "(CancelExpression _ _)");
    test_parse!(cancelexpr_value: "cancel true" => CancelExpression::parse => "(CancelExpression _ _)");
    test_parse_error!(cancelexpr_empty: "cancel" => CancelExpression::parse);
}
