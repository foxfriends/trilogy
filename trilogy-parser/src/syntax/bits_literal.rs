use super::*;
use crate::Parser;
use bitvec::prelude::*;
use trilogy_scanner::{Token, TokenType};

/// A bits literal expression
///
/// ```trilogy
/// 0bb1010
/// ```
#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct BitsLiteral {
    pub token: Token,
}

impl BitsLiteral {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(TokenType::Bits)
            .map_err(|token| parser.expected(token, "expected bits literal"))?;
        Ok(Self { token })
    }

    /// The bits value of this expression.
    pub fn value(&self) -> &BitVec<usize, Msb0> {
        self.token.value.as_ref().unwrap().as_bits().unwrap()
    }

    /// The bits value of this expression.
    pub fn into_value(self) -> BitVec<usize, Msb0> {
        self.token.value.unwrap().into_bits().unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(bits_bin: "0bb0101" => BitsLiteral::parse => "(BitsLiteral _)");
    test_parse!(bits_hex: "0xb10af" => BitsLiteral::parse => "(BitsLiteral _)");
    test_parse!(bits_oct: "0ob107" => BitsLiteral::parse => "(BitsLiteral _)");
    test_parse_error!(not_bits: "0b101" => BitsLiteral::parse => "expected bits literal");
}
