use super::{
    Array, Atom, Bits, Continuation, Number, Procedure, Record, ReferentialEq, Set, Struct,
    StructuralEq, Tuple,
};
use num::ToPrimitive;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Rem, Shl, Shr, Sub};

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
    Procedure(Procedure),
    Continuation(Continuation),
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

impl Neg for Value {
    type Output = Result<Value, ()>;

    fn neg(self) -> Self::Output {
        match self {
            Self::Number(val) => Ok(Self::Number(-val)),
            _ => Err(()),
        }
    }
}

impl BitAnd for Value {
    type Output = Result<Value, ()>;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Bits(lhs), Self::Bits(rhs)) => Ok(Self::Bits(lhs & rhs)),
            _ => Err(()),
        }
    }
}

impl BitOr for Value {
    type Output = Result<Value, ()>;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Bits(lhs), Self::Bits(rhs)) => Ok(Self::Bits(lhs | rhs)),
            _ => Err(()),
        }
    }
}

impl BitXor for Value {
    type Output = Result<Value, ()>;

    fn bitxor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Bits(lhs), Self::Bits(rhs)) => Ok(Self::Bits(lhs ^ rhs)),
            _ => Err(()),
        }
    }
}

impl Shl for Value {
    type Output = Result<Value, ()>;

    fn shl(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Bits(lhs), Self::Number(rhs)) if rhs.is_integer() => Ok(Value::Bits(
                lhs << rhs.as_integer().ok_or(())?.to_usize().ok_or(())?,
            )),
            _ => Err(()),
        }
    }
}

impl Shr for Value {
    type Output = Result<Value, ()>;

    fn shr(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Bits(lhs), Self::Number(rhs)) if rhs.is_integer() => Ok(Value::Bits(
                lhs >> rhs.as_integer().ok_or(())?.to_usize().ok_or(())?,
            )),
            _ => Err(()),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Number(lhs), Self::Number(rhs)) => lhs.partial_cmp(rhs),
            (Self::Bool(lhs), Self::Bool(rhs)) => lhs.partial_cmp(rhs),
            (Self::Char(lhs), Self::Char(rhs)) => lhs.partial_cmp(rhs),
            (Self::String(lhs), Self::String(rhs)) => lhs.partial_cmp(rhs),
            (Self::Struct(lhs), Self::Struct(rhs)) => lhs.partial_cmp(rhs),
            (Self::Bits(lhs), Self::Bits(rhs)) => lhs.partial_cmp(rhs),
            (Self::Tuple(lhs), Self::Tuple(rhs)) => lhs.partial_cmp(rhs),
            (Self::Array(lhs), Self::Array(rhs)) => lhs.partial_cmp(rhs),
            _ => None,
        }
    }
}
