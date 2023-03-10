use super::{Array, Atom, Bits, Number, Record, ReferentialEq, Set, Struct, StructuralEq, Tuple};
use std::ops::{Add, Div, Mul, Rem, Sub};

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum Value {
    Unit,
    Bool(bool),
    Char(char),
    String(String),
    Number(Number),
    Bits(Bits),
    Atom(Atom),
    Struct(Struct),
    Tuple(Tuple),
    Array(Array),
    Set(Set),
    Record(Record),

    // TODO: these will require some thought
    Function,
    Procedure,
    Rule,
    Module,
    Continuation,

    // Due to unification, any value may or may not be fully instantiated.
    // Patterns at runtime are just values with holes.
    Hole,
}

impl ReferentialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Array(lhs), Self::Array(rhs)) => ReferentialEq::eq(lhs, rhs),
            (Self::Record(lhs), Self::Record(rhs)) => ReferentialEq::eq(lhs, rhs),
            (Self::Set(lhs), Self::Set(rhs)) => ReferentialEq::eq(lhs, rhs),
            _ => self == other,
        }
    }
}

impl StructuralEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Array(lhs), Self::Array(rhs)) => StructuralEq::eq(lhs, rhs),
            (Self::Record(lhs), Self::Record(rhs)) => StructuralEq::eq(lhs, rhs),
            (Self::Set(lhs), Self::Set(rhs)) => StructuralEq::eq(lhs, rhs),
            _ => self == other,
        }
    }
}

impl Add for Value {
    type Output = Result<Value, ()>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs + rhs)),
            _ => Err(()),
        }
    }
}

impl Sub for Value {
    type Output = Result<Value, ()>;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs - rhs)),
            _ => Err(()),
        }
    }
}

impl Mul for Value {
    type Output = Result<Value, ()>;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs * rhs)),
            _ => Err(()),
        }
    }
}

impl Div for Value {
    type Output = Result<Value, ()>;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs / rhs)),
            _ => Err(()),
        }
    }
}

impl Rem for Value {
    type Output = Result<Value, ()>;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs % rhs)),
            _ => Err(()),
        }
    }
}
