use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

/// A break statement.
///
/// ```trilogy
/// break
/// ```
#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct BreakStatement {
    pub r#break: Token,
}

impl BreakStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#break = parser
            .expect(TokenType::KwBreak)
            .expect("Caller should have found this");
        Ok(Self { r#break })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(breakstmt_empty: "break" => BreakStatement::parse => "(BreakStatement _)");
    test_parse_error!(breakstmt_value: "break unit" => BreakStatement::parse);
}
