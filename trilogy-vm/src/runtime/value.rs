use super::callable::{Continuation, Native, Procedure};
use super::{
    Array, Atom, Bits, Callable, Number, Record, ReferentialEq, Set, String, Struct, StructuralEq,
    Tuple,
};
use crate::callable::Closure;
use bitvec::prelude::*;
use num::ToPrimitive;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Rem, Shl, Shr, Sub};

/// Generic value type, encapsulating every type of value that can be handled by
/// the Trilogy Virtual Machine.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum Value {
    Unit,
    Bool(bool),
    Char(char),
    String(String),
    Number(Number),
    Bits(Bits),
    #[serde(skip_deserializing)]
    Atom(Atom),
    #[serde(skip_deserializing)]
    Struct(Struct),
    #[serde(skip_deserializing)]
    Tuple(Tuple),
    Array(Array),
    #[serde(skip_deserializing)]
    Set(Set),
    Record(Record),
    #[serde(skip)]
    Callable(Callable),
}

impl Value {
    /// Returns true if the `Value` is `unit`. Returns false otherwise.
    ///
    /// ```
    /// # use trilogy_vm::Value;
    /// assert!(Value::Unit.is_unit());
    /// assert!(!Value::Bool(false).is_unit());
    /// ```
    pub fn is_unit(&self) -> bool {
        matches!(self, Value::Unit)
    }

    /// If the `Value` is `unit`, returns a `()`. Returns None otherwise.
    ///
    /// ```
    /// # use trilogy_vm::Value;
    /// assert_eq!(Value::Unit.as_unit(), Some(()));
    /// assert_eq!(Value::Bool(false).as_unit(), None);
    /// ```
    pub fn as_unit(&self) -> Option<()> {
        match self {
            Value::Unit => Some(()),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a boolean. Returns false otherwise.
    ///
    /// ```
    /// # use trilogy_vm::Value;
    /// assert!(Value::Bool(false).is_bool());
    /// assert!(!Value::Unit.is_bool());
    /// ```
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(..))
    }

    /// If the `Value` is a boolean, returns the boolean value. Returns None otherwise.
    ///
    /// ```
    /// # use trilogy_vm::Value;
    /// assert_eq!(Value::Bool(false).as_bool(), Some(false));
    /// assert_eq!(Value::Unit.as_bool(), None);
    /// ```
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a char. Returns false otherwise.
    ///
    /// ```
    /// # use trilogy_vm::Value;
    /// assert!(Value::Char('a').is_char());
    /// assert!(!Value::Unit.is_char());
    /// ```
    pub fn is_char(&self) -> bool {
        matches!(self, Value::Char(..))
    }

    /// If the `Value` is a char, returns the char value. Returns None otherwise.
    ///
    /// ```
    /// # use trilogy_vm::Value;
    /// assert_eq!(Value::Char('a').as_char(), Some('a'));
    /// assert_eq!(Value::Unit.as_char(), None);
    /// ```
    pub fn as_char(&self) -> Option<char> {
        match self {
            Value::Char(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a string. Returns false otherwise.
    ///
    /// ```
    /// # use trilogy_vm::Value;
    /// assert!(Value::from("hello world").is_string());
    /// assert!(!Value::Unit.is_string());
    /// ```
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(..))
    }

    /// If the `Value` is a String, returns the str value. Returns None otherwise.
    ///
    /// ```
    /// # use trilogy_vm::Value;
    /// assert_eq!(Value::from("hello world").as_str(), Some("hello world"));
    /// assert_eq!(Value::Unit.as_str(), None);
    /// ```
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(value) => Some(value.as_ref()),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a number. Returns false otherwise.
    ///
    /// ```
    /// # use trilogy_vm::{Value, Number};
    /// assert!(Value::from(Number::from(1)).is_number());
    /// assert!(!Value::Unit.is_number());
    /// ```
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(..))
    }

    /// If the `Value` is a Number, returns the number value. Returns None otherwise.
    ///
    /// Note that the return [`Number`][] is still a Trilogy number, and so is capable
    /// of representing arbitrary precision real numbers and imaginary numbers. Further
    /// conversion to a Rust number type is likely necessary.
    ///
    /// ```
    /// # use trilogy_vm::{Value, Number};
    /// assert_eq!(Value::from(Number::from(1)).as_number(), Some(&Number::from(1)));
    /// assert_eq!(Value::Unit.as_number(), None);
    /// ```
    pub fn as_number(&self) -> Option<&Number> {
        match self {
            Value::Number(value) => Some(value),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a bits. Returns false otherwise.
    ///
    /// ```
    /// # use trilogy_vm::{Value, Bits};
    /// assert!(Value::from(Bits::from_iter(b"123")).is_bits());
    /// assert!(!Value::Unit.is_bits());
    /// ```
    pub fn is_bits(&self) -> bool {
        matches!(self, Value::Bits(..))
    }

    /// If the `Value` is a Bits, returns the bits value. Returns None otherwise.
    ///
    /// ```
    /// # use trilogy_vm::{Value, Bits};
    /// assert_eq!(Value::from(Bits::from_iter(b"123")).as_bits(), Some(&Bits::from_iter(b"123")));
    /// assert_eq!(Value::Unit.as_bits(), None);
    /// ```
    pub fn as_bits(&self) -> Option<&Bits> {
        match self {
            Value::Bits(value) => Some(value),
            _ => None,
        }
    }

    /// Returns true if the `Value` is an atom. Returns false otherwise.
    ///
    /// Note that atoms are tied to an instance of the `VirtualMachine`, and cannot
    /// be created in isolation.
    ///
    /// ```
    /// # use trilogy_vm::{Value, VirtualMachine};
    /// let vm = VirtualMachine::new();
    /// let atom = vm.atom("atom");
    /// assert_eq!(Value::Atom(atom).is_atom(), true);
    /// assert_eq!(Value::Unit.is_atom(), false);
    /// ```
    pub fn is_atom(&self) -> bool {
        matches!(self, Value::Atom(..))
    }

    /// If the `Value` is an Atom, returns the atom. Returns None otherwise.
    ///
    /// Note that atoms are tied to an instance of the `VirtualMachine`, and cannot
    /// be created in isolation.
    ///
    /// ```
    /// # use trilogy_vm::{Value, VirtualMachine};
    /// let vm = VirtualMachine::new();
    /// let atom = vm.atom("atom");
    /// assert_eq!(Value::Atom(atom.clone()).as_atom(), Some(&atom));
    /// assert_eq!(Value::Unit.as_atom(), None);
    /// ```
    pub fn as_atom(&self) -> Option<&Atom> {
        match self {
            Value::Atom(value) => Some(value),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a Trilogy struct. Returns false otherwise.
    ///
    /// ```
    /// # use trilogy_vm::{Value, Struct, VirtualMachine};
    /// let vm = VirtualMachine::new();
    /// let val = Struct::new(vm.atom("mystruct"), Value::Unit);
    /// assert_eq!(Value::Struct(val.clone()).is_struct(), true);
    /// assert_eq!(Value::Unit.is_struct(), false);
    /// ```
    pub fn is_struct(&self) -> bool {
        matches!(self, Value::Struct(..))
    }

    /// If the `Value` is a Trilogy struct, returns the struct value. Returns None otherwise.
    ///
    /// ```
    /// # use trilogy_vm::{Value, Struct, VirtualMachine};
    /// let vm = VirtualMachine::new();
    /// let val = Struct::new(vm.atom("mystruct"), Value::Unit);
    /// assert_eq!(Value::Struct(val.clone()).as_struct(), Some(&val));
    /// assert_eq!(Value::Unit.as_struct(), None);
    /// ```
    pub fn as_struct(&self) -> Option<&Struct> {
        match self {
            Value::Struct(value) => Some(value),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a tuple. Returns false otherwise.
    ///
    /// ```
    /// # use trilogy_vm::{Value, Tuple};
    /// assert_eq!(Value::Tuple(Tuple::from((1, 2))).is_tuple(), true);
    /// assert_eq!(Value::Unit.is_tuple(), false);
    /// ```
    pub fn is_tuple(&self) -> bool {
        matches!(self, Value::Tuple(..))
    }

    /// If the `Value` is a tuple, returns the tuple value. Returns None otherwise.
    ///
    ///
    /// ```
    /// # use trilogy_vm::{Value, Tuple};
    /// assert_eq!(Value::Tuple(Tuple::from((1, 2))).as_tuple(), Some(&Tuple::from((1, 2))));
    /// assert_eq!(Value::Unit.as_tuple(), None);
    /// ```
    pub fn as_tuple(&self) -> Option<&Tuple> {
        match self {
            Value::Tuple(value) => Some(value),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a set. Returns false otherwise.
    ///
    /// ```
    /// # use trilogy_vm::{Value, Set};
    /// assert_eq!(Value::Set(Set::new()).is_set(), true);
    /// assert_eq!(Value::Unit.is_set(), false);
    /// ```
    pub fn is_set(&self) -> bool {
        matches!(self, Value::Set(..))
    }

    /// If the `Value` is a set, returns the set value. Returns None otherwise.
    ///
    /// ```
    /// # use trilogy_vm::{Value, Set};
    /// let set = Set::new();
    /// assert_eq!(Value::Set(set.clone()).as_set(), Some(&set));
    /// assert_eq!(Value::Unit.as_set(), None);
    /// ```
    pub fn as_set(&self) -> Option<&Set> {
        match self {
            Value::Set(value) => Some(value),
            _ => None,
        }
    }

    /// Returns true if the `Value` is an array. Returns false otherwise.
    ///
    /// ```
    /// # use trilogy_vm::{Value, Array};
    /// assert_eq!(Value::Array(Array::new()).is_array(), true);
    /// assert_eq!(Value::Unit.is_array(), false);
    /// ```
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(..))
    }

    /// If the `Value` is an array, returns the array value. Returns None otherwise.
    ///
    /// ```
    /// # use trilogy_vm::{Value, Array};
    /// let array = Array::new();
    /// assert_eq!(Value::Array(array.clone()).as_array(), Some(&array));
    /// assert_eq!(Value::Unit.as_array(), None);
    /// ```
    pub fn as_array(&self) -> Option<&Array> {
        match self {
            Value::Array(value) => Some(value),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a record. Returns false otherwise.
    ///
    /// ```
    /// # use trilogy_vm::{Value, Record};
    /// assert_eq!(Value::Record(Record::new()).is_record(), true);
    /// assert_eq!(Value::Unit.is_record(), false);
    /// ```
    pub fn is_record(&self) -> bool {
        matches!(self, Value::Record(..))
    }

    /// If the `Value` is a record, returns the record value. Returns None otherwise.
    ///
    /// ```
    /// # use trilogy_vm::{Value, Record};
    /// let array = Record::new();
    /// assert_eq!(Value::Record(array.clone()).as_record(), Some(&array));
    /// assert_eq!(Value::Unit.as_record(), None);
    /// ```
    pub fn as_record(&self) -> Option<&Record> {
        match self {
            Value::Record(value) => Some(value),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a callable. Returns false otherwise.
    pub fn is_callable(&self) -> bool {
        matches!(self, Value::Callable(..))
    }

    /// If the `Value` is a callable, returns the callable value. Returns None otherwise.
    pub fn as_callable(&self) -> Option<&Callable> {
        match self {
            Value::Callable(value) => Some(value),
            _ => None,
        }
    }
}

impl Value {
    /// Performs a deep clone of this value.
    ///
    /// This is only relevant for compound types, for which the deep clone
    /// means that every value within the container is recursively cloned,
    /// such that the new value is completely disjoint from the original.
    /// As that sounds, this is potentially a very slow and expensive
    /// operation.
    pub fn structural_clone(&self) -> Self {
        match self {
            Self::Array(array) => Self::Array(array.structural_clone()),
            Self::Set(array) => Self::Set(array.structural_clone()),
            Self::Record(array) => Self::Record(array.structural_clone()),
            _ => self.clone(),
        }
    }

    /// Performs a shallow clone of this value.
    ///
    /// This is only relevant for compound types, for which the shallow clone
    /// means that the container is cloned but the values contained remain
    /// references to the same values as the original container.
    pub fn shallow_clone(&self) -> Self {
        match self {
            Self::Array(array) => Self::Array(array.shallow_clone()),
            Self::Set(array) => Self::Set(array.shallow_clone()),
            Self::Record(array) => Self::Record(array.shallow_clone()),
            _ => self.clone(),
        }
    }
}

impl ReferentialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Array(lhs), Self::Array(rhs)) => ReferentialEq::eq(lhs, rhs),
            (Self::Set(lhs), Self::Set(rhs)) => ReferentialEq::eq(lhs, rhs),
            (Self::Record(lhs), Self::Record(rhs)) => ReferentialEq::eq(lhs, rhs),
            (Self::Tuple(lhs), Self::Tuple(rhs)) => ReferentialEq::eq(lhs, rhs),
            (Self::Struct(lhs), Self::Struct(rhs)) => ReferentialEq::eq(lhs, rhs),
            _ => self == other,
        }
    }
}

impl StructuralEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Array(lhs), Self::Array(rhs)) => StructuralEq::eq(lhs, rhs),
            (Self::Set(lhs), Self::Set(rhs)) => StructuralEq::eq(lhs, rhs),
            (Self::Record(lhs), Self::Record(rhs)) => StructuralEq::eq(lhs, rhs),
            (Self::Tuple(lhs), Self::Tuple(rhs)) => StructuralEq::eq(lhs, rhs),
            (Self::Struct(lhs), Self::Struct(rhs)) => StructuralEq::eq(lhs, rhs),
            _ => self == other,
        }
    }
}

impl Add for Value {
    type Output = Result<Value, ()>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::from(lhs + rhs)),
            _ => Err(()),
        }
    }
}

impl Sub for Value {
    type Output = Result<Value, ()>;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::from(lhs - rhs)),
            _ => Err(()),
        }
    }
}

impl Mul for Value {
    type Output = Result<Value, ()>;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::from(lhs * rhs)),
            _ => Err(()),
        }
    }
}

impl Div for Value {
    type Output = Result<Value, ()>;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::from(lhs / rhs)),
            _ => Err(()),
        }
    }
}

impl Rem for Value {
    type Output = Result<Value, ()>;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::from(lhs % rhs)),
            _ => Err(()),
        }
    }
}

impl Neg for Value {
    type Output = Result<Value, ()>;

    fn neg(self) -> Self::Output {
        match self {
            Self::Number(val) => Ok(Self::from(-val)),
            _ => Err(()),
        }
    }
}

impl BitAnd for Value {
    type Output = Result<Value, ()>;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Bits(lhs), Self::Bits(rhs)) => Ok(Self::from(lhs & rhs)),
            _ => Err(()),
        }
    }
}

impl BitOr for Value {
    type Output = Result<Value, ()>;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Bits(lhs), Self::Bits(rhs)) => Ok(Self::from(lhs | rhs)),
            _ => Err(()),
        }
    }
}

impl BitXor for Value {
    type Output = Result<Value, ()>;

    fn bitxor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Bits(lhs), Self::Bits(rhs)) => Ok(Self::from(lhs ^ rhs)),
            _ => Err(()),
        }
    }
}

impl Shl for Value {
    type Output = Result<Value, ()>;

    fn shl(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Bits(lhs), Self::Number(rhs)) if rhs.is_integer() => Ok(Value::from(
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
            (Self::Bits(lhs), Self::Number(rhs)) if rhs.is_integer() => Ok(Value::from(
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
            (Self::String(lhs), Self::String(rhs)) => lhs.as_ref().partial_cmp(rhs.as_ref()),
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
            Self::Callable(value) => write!(f, "{value}"),
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

    (<$fromty:ty> for $variant:ident via $via:ty) => {
        impl From<$fromty> for Value {
            fn from(value: $fromty) -> Self {
                Self::$variant(<$via>::from(value))
            }
        }
    };
}

impl_from!(<String> for String via String);
impl_from!(<std::string::String> for String via String);
impl_from!(<&str> for String via String);
impl_from!(<&String> for String via String);
impl_from!(<&std::string::String> for String via String);
impl_from!(<Number> for Number via Number);
impl_from!(<char> for Char);
impl_from!(<bool> for Bool);
impl_from!(<Bits> for Bits via Bits);
impl_from!(<Vec<bool>> for Bits via Bits);
impl_from!(<BitVec<usize, Msb0>> for Bits via Bits);
impl_from!(<Atom> for Atom);
impl_from!(<Struct> for Struct);
impl_from!(<Set> for Set);
impl_from!(<Record> for Record);
impl_from!(<Array> for Array);
impl_from!(<Tuple> for Tuple);
impl_from!(<HashMap<Value, Value>> for Record via Record);
impl_from!(<HashSet<Value>> for Set via Set);
impl_from!(<Vec<Value>> for Array via Array);
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
impl_from!(<f32> for Number via Number);
impl_from!(<f64> for Number via Number);
impl_from!(<num::BigRational> for Number via Number);
impl_from!(<num::BigInt> for Number via Number);
impl_from!(<num::BigUint> for Number via Number);
impl_from!(<num::Complex<num::BigRational>> for Number via Number);
impl_from!(<Callable> for Callable);
impl_from!(<Procedure> for Callable via Callable);
impl_from!(<Closure> for Callable via Callable);
impl_from!(<Continuation> for Callable via Callable);
impl_from!(<Native> for Callable via Callable);

#[cfg(feature = "sqlx")]
impl_from!(<sqlx::types::BigDecimal> for Number via Number);

#[cfg(feature = "sqlx")]
impl_from!(<sqlx::types::BitVec> for Bits via Bits);

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Self::Unit
    }
}

impl<T, U> From<(T, U)> for Value
where
    Value: From<T>,
    Value: From<U>,
{
    fn from(value: (T, U)) -> Self {
        Self::Tuple(Tuple::from(value))
    }
}

macro_rules! impl_into {
    (<$intoty:ty> via $variant:ident as $id:ident => $expr:expr) => {
        impl TryFrom<Value> for $intoty {
            type Error = Value;

            fn try_from(value: Value) -> Result<Self, Self::Error> {
                match value {
                    Value::$variant($id) => $expr,
                    value => Err(value),
                }
            }
        }
    };

    (<$intoty:ty> via $variant:ident) => {
        impl_into!(<$intoty> via $variant as into => Ok(into.into()));
    };

    (try <$intoty:ty> via $variant:ident) => {
        impl_into!(<$intoty> via $variant as into => into.try_into().map_err(Value::from));
    };
}

impl_into!(<String> via String);
impl_into!(<std::string::String> via String);
impl_into!(<Number> via Number);
impl_into!(try <usize> via Number);
impl_into!(try <u8> via Number);
impl_into!(try <u16> via Number);
impl_into!(try <u32> via Number);
impl_into!(try <u64> via Number);
impl_into!(try <u128> via Number);
impl_into!(try <isize> via Number);
impl_into!(try <i8> via Number);
impl_into!(try <i16> via Number);
impl_into!(try <i32> via Number);
impl_into!(try <i64> via Number);
impl_into!(try <i128> via Number);
impl_into!(<num::Complex<num::BigRational>> via Number);
impl_into!(try <num::BigRational> via Number);
impl_into!(try <num::BigInt> via Number);
impl_into!(try <num::BigUint> via Number);
impl_into!(<char> via Char);
impl_into!(<bool> via Bool);
impl_into!(<Bits> via Bits);
impl_into!(<Atom> via Atom);
impl_into!(<Struct> via Struct);
impl_into!(<Set> via Set);
impl_into!(<Record> via Record);
impl_into!(<Array> via Array);
impl_into!(<Tuple> via Tuple);
impl_into!(<Callable> via Callable);
impl_into!(<Vec<Value>> via Array);
impl_into!(<HashMap<Value, Value>> via Record);
impl_into!(<HashSet<Value>> via Set);

impl TryFrom<Value> for () {
    type Error = Value;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Unit => Ok(()),
            value => Err(value),
        }
    }
}

impl<T> From<Option<T>> for Value
where
    Value: From<T>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            None => Self::Unit,
            Some(value) => Self::from(value),
        }
    }
}

#[cfg(feature = "json")]
impl From<serde_json::Value> for Value {
    fn from(value: serde_json::Value) -> Value {
        match value {
            serde_json::Value::Null => Value::Unit,
            serde_json::Value::Bool(v) => Value::Bool(v),
            serde_json::Value::Number(v) => Value::Number(v.into()),
            serde_json::Value::String(v) => Value::String(v.into()),
            serde_json::Value::Array(v) => Value::Array(v.into_iter().map(Value::from).collect()),
            serde_json::Value::Object(v) => Value::Record(
                v.into_iter()
                    .map(|(k, v)| (Value::from(k), Value::from(v)))
                    .collect(),
            ),
        }
    }
}
