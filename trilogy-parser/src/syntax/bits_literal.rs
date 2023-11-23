use super::*;
use crate::Parser;
use bitvec::prelude::*;
use trilogy_scanner::{Token, TokenType, TokenValue};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct BitsLiteral {
    token: Token,
}

impl BitsLiteral {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(TokenType::Bits)
            .map_err(|token| parser.expected(token, "expected bits literal"))?;
        Ok(Self { token })
    }

    pub fn value(&self) -> BitVec<usize, Msb0> {
        let TokenValue::Bits(bits) = self.token.value.as_ref().unwrap() else {
            unreachable!()
        };
        bits.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(bits_bin: "0bb0101" => BitsLiteral::parse => "(BitsLiteral)");
    test_parse!(bits_hex: "0xb10af" => BitsLiteral::parse => "(BitsLiteral)");
    test_parse!(bits_oct: "0ob107" => BitsLiteral::parse => "(BitsLiteral)");
    test_parse_error!(not_bits: "0b101" => BitsLiteral::parse => "expected bits literal");
}
