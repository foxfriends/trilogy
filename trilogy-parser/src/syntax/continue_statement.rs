use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

/// A continue statement.
///
/// ```trilogy
/// continue
/// ```
#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ContinueStatement {
    pub r#continue: Token,
}

impl ContinueStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#continue = parser
            .expect(TokenType::KwContinue)
            .expect("Caller should have found this");
        Ok(Self { r#continue })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(continuestmt_empty: "continue" => ContinueStatement::parse => "(ContinueStatement _)");
    test_parse_error!(continuestmt_value: "continue unit" => ContinueStatement::parse);
}
