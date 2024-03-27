use super::RefCount;
use std::fmt::{self, Debug, Display};
use std::ops::{Add, Deref};

#[derive(Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct String(RefCount<std::string::String>);

impl Display for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Debug for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl AsRef<str> for String {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<String> for std::string::String {
    fn from(value: String) -> Self {
        (*value.0).to_owned()
    }
}

impl From<std::string::String> for String {
    fn from(value: std::string::String) -> Self {
        Self(RefCount::new(value))
    }
}

impl From<&std::string::String> for String {
    fn from(value: &std::string::String) -> Self {
        Self(RefCount::new(value.into()))
    }
}

impl From<&String> for String {
    fn from(value: &String) -> Self {
        value.clone()
    }
}

impl From<&str> for String {
    fn from(value: &str) -> Self {
        Self(RefCount::new(value.to_owned()))
    }
}

impl Deref for String {
    type Target = std::string::String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C> FromIterator<C> for String
where
    std::string::String: FromIterator<C>,
{
    fn from_iter<T: IntoIterator<Item = C>>(iter: T) -> Self {
        Self(RefCount::new(iter.into_iter().collect()))
    }
}

impl<T> Add<T> for String
where
    std::string::String: Add<T, Output = std::string::String>,
{
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        Self::from((*self.0).clone() + rhs)
    }
}
