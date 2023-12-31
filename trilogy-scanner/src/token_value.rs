use bitvec::prelude::*;
use num::{rational::BigRational, Complex};

/// The raw value a token represents.
///
/// The exact interpretation of this value is dependent on the
/// type of token being parsed.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TokenValue {
    /// A character value, having parsed any escape sequences.
    Char(char),
    /// A string valued token.
    ///
    /// For string literals, this value represents the value of the string, having parsed
    /// any escape sequences.
    ///
    /// A string value is also used to represent the actual contents of an atom or identifier
    /// token.
    String(String),
    // Complex<BigRational> is kind of large, so Box just to make clippy quiet for now.
    // Will require some tuning probably... Maybe storing integers/real numbers in
    // smaller data types since they are the most common types of numbers anyway
    // might be worthwhile.
    /// A number value, having been parsed and converted to an actual number.
    Number(Box<Complex<BigRational>>),
    /// A bits value, having been parsed into an actual binary sequence.
    Bits(BitVec<usize, Msb0>),
}

impl TokenValue {
    /// The string value of this token, if any.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(string) => Some(string),
            _ => None,
        }
    }

    /// The bits value of this token, if any.
    #[must_use]
    pub fn as_bits(&self) -> Option<&BitVec<usize, Msb0>> {
        match self {
            Self::Bits(bits) => Some(bits),
            _ => None,
        }
    }

    /// Consumes this token value, returning the underlying bits value. If the
    /// token does not have a bits value, it is discarded.
    #[must_use]
    pub fn into_bits(self) -> Option<BitVec<usize, Msb0>> {
        match self {
            Self::Bits(bits) => Some(bits),
            _ => None,
        }
    }
}

impl From<String> for TokenValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl TryFrom<TokenValue> for String {
    type Error = ();

    fn try_from(value: TokenValue) -> Result<Self, ()> {
        match value {
            TokenValue::String(value) => Ok(value),
            _ => Err(()),
        }
    }
}

impl From<char> for TokenValue {
    fn from(value: char) -> Self {
        Self::Char(value)
    }
}

impl From<&'static str> for TokenValue {
    fn from(value: &'static str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<Complex<BigRational>> for TokenValue {
    fn from(value: Complex<BigRational>) -> Self {
        Self::Number(Box::new(value))
    }
}

impl From<BitVec<usize, Msb0>> for TokenValue {
    fn from(value: BitVec<usize, Msb0>) -> Self {
        Self::Bits(value)
    }
}
