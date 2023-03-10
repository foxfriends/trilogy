use bitvec::vec::BitVec;
use num::{rational::BigRational, Complex};

/// The raw value a token represents.
///
/// The exact interpretation of this value is dependent on the
/// type of token being parsed.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TokenValue {
    Char(char),
    String(String),
    // Complex<BigRational> is kind of large, so Box just to make clippy quiet for now.
    // Will require some tuning probably... Maybe storing integers/real numbers in
    // smaller data types since they are the most common types of numbers anyway
    // might be worthwhile.
    Number(Box<Complex<BigRational>>),
    Bits(BitVec),
}

impl TokenValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(string) => Some(string),
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

impl From<BitVec> for TokenValue {
    fn from(value: BitVec) -> Self {
        Self::Bits(value)
    }
}
