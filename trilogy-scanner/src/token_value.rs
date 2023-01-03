use num::{rational::BigRational, BigUint};

/// The raw value a token represents.
///
/// The exact interpretation of this value is dependent on the
/// type of token being parsed.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TokenValue {
    Char(char),
    String(String),
    Integer(BigUint),
    Rational(BigRational),
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
