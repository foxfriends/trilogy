use super::{expression::Precedence, *};
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

/// An exit expression.
///
/// ```trilogy
/// exit 123
/// ```
#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ExitExpression {
    pub exit: Token,
    pub expression: Expression,
}

impl ExitExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let exit = parser
            .expect(KwExit)
            .expect("Caller should have found this");
        let expression = Expression::parse_precedence(parser, Precedence::Continuation)?;
        Ok(Self { exit, expression })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(exitexpr_unit: "exit unit" => ExitExpression::parse => "(ExitExpression _ _)");
    test_parse!(exitexpr_value: "exit true" => ExitExpression::parse => "(ExitExpression _ _)");
    test_parse_error!(exitexpr_empty: "exit" => ExitExpression::parse);
}
