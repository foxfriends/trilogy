use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType, TokenValue};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct AtomLiteral {
    token: Token,
}

impl AtomLiteral {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(TokenType::Atom)
            .map_err(|token| parser.expected(token, "expected atom literal"))?;
        Ok(Self { token })
    }

    pub fn value(&self) -> String {
        let TokenValue::String(value) = self.token.value.as_ref().unwrap() else { unreachable!() };
        value.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(atom: "'hello" => AtomLiteral::parse => "(AtomLiteral)");
    test_parse_error!(not_atom: "hello" => AtomLiteral::parse => "expected atom literal");
}
