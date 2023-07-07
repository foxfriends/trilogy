use super::{
    Array, Atom, Bits, Continuation, Number, Procedure, Record, ReferentialEq, Set, Struct,
    StructuralEq, Tuple,
};
use num::ToPrimitive;
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display};
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Rem, Shl, Shr, Sub};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
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

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unit => write!(f, "unit"),
            Self::Bool(value) => write!(f, "{value}"),
            Self::Char(value) => write!(f, "{value:?}"), // TODO: officially implement
            Self::String(value) => write!(f, "{value:?}"), // TODO: officially implement
            Self::Number(value) => write!(f, "{value}"),
            Self::Bits(value) => write!(f, "{value}"),
            Self::Atom(value) => write!(f, "{value}"),
            Self::Struct(value) => write!(f, "{value}"),
            Self::Tuple(value) => write!(f, "{value}"),
            Self::Array(value) => write!(f, "{value}"),
            Self::Set(value) => write!(f, "{value}"),
            Self::Record(value) => write!(f, "{value}"),
            Self::Procedure(value) => write!(f, "{value}"),
            Self::Continuation(..) => Err(fmt::Error),
        }
    }
}

macro_rules! impl_from {
    (<$fromty:ty> for $variant:ident) => {
        impl From<$fromty> for Value {
            fn from(value: $fromty) -> Self {
                Self::$variant(value)
            }
        }
    };

    (<$fromty:ty> for $variant:ident via $via:ident) => {
        impl From<$fromty> for Value {
            fn from(value: $fromty) -> Self {
                Self::$variant($via::from(value))
            }
        }
    };
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Self::Unit
    }
}

impl_from!(<String> for String);
impl_from!(<Number> for Number);
impl_from!(<char> for Char);
impl_from!(<bool> for Bool);
impl_from!(<Bits> for Bits);
impl_from!(<Atom> for Atom);
impl_from!(<Struct> for Struct);
impl_from!(<Set> for Set);
impl_from!(<Record> for Record);
impl_from!(<Array> for Array);
impl_from!(<Tuple> for Tuple);
impl_from!(<Procedure> for Procedure);
impl_from!(<Continuation> for Continuation);

impl<T, U> From<(T, U)> for Value
where
    Value: From<T>,
    Value: From<U>,
{
    fn from(value: (T, U)) -> Self {
        Self::Tuple(Tuple::from(value))
    }
}

impl_from!(<HashMap<Value, Value>> for Record via Record);
impl_from!(<HashSet<Value>> for Set via Set);
impl_from!(<Vec<Value>> for Array via Array);
impl_from!(<Vec<bool>> for Bits via Bits);
impl_from!(<&str> for String via String);
impl_from!(<usize> for Number via Number);
impl_from!(<u8> for Number via Number);
impl_from!(<u16> for Number via Number);
impl_from!(<u32> for Number via Number);
impl_from!(<u64> for Number via Number);
impl_from!(<u128> for Number via Number);
impl_from!(<isize> for Number via Number);
impl_from!(<i8> for Number via Number);
impl_from!(<i16> for Number via Number);
impl_from!(<i32> for Number via Number);
impl_from!(<i64> for Number via Number);
impl_from!(<i128> for Number via Number);
impl_from!(<num::BigRational> for Number via Number);
impl_from!(<num::BigInt> for Number via Number);
impl_from!(<num::BigUint> for Number via Number);
impl_from!(<num::Complex<num::BigRational>> for Number via Number);
