use super::RefCount;
use num::complex::ParseComplexError;
use num::rational::ParseRatioError;
use num::traits::Pow;
use num::{BigInt, BigRational, BigUint, Complex, FromPrimitive, One, ToPrimitive, Zero};
use std::fmt::{self, Display};
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};
use std::str::FromStr;

/// A Trilogy Number value.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Number(RefCount<Complex<BigRational>>);

#[cfg(feature = "serde")]
impl serde::Serialize for Number {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Ok(int) = self.try_into() {
            serializer.serialize_i64(int)
        } else if let Ok(int) = self.try_into() {
            serializer.serialize_u64(int)
        } else if let Ok(float) = self.try_into() {
            serializer.serialize_f64(float)
        } else {
            serializer.serialize_str(&self.to_string())
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Number {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum IntOrFloat {
            UInt(u64),
            Int(i64),
            Float(f64),
        }
        match IntOrFloat::deserialize(deserializer)? {
            IntOrFloat::UInt(int) => Ok(Self::from(int)),
            IntOrFloat::Int(int) => Ok(Self::from(int)),
            IntOrFloat::Float(float) => Ok(Self::from(float)),
        }
    }
}

macro_rules! proxy_op_int_opt {
    ($t:ty, $f:ident) => {
        impl $t for Number {
            type Output = Self;

            fn $f(self, rhs: Self) -> Self::Output {
                if let (Some(lhs), Some(rhs)) = (self.as_integer(), rhs.as_integer()) {
                    Self::from(lhs.$f(rhs))
                } else if let (Some(lhs), Some(rhs)) = (self.as_real(), rhs.as_real()) {
                    Self::from(lhs.$f(rhs))
                } else {
                    Self::from((*self.0).clone().$f(&*rhs.0))
                }
            }
        }
    };
}

macro_rules! proxy_op {
    ($t:ty, $f:ident) => {
        impl $t for Number {
            type Output = Self;

            fn $f(self, rhs: Self) -> Self::Output {
                if let (Some(lhs), Some(rhs)) = (self.as_real(), rhs.as_real()) {
                    Self::from(lhs.$f(rhs))
                } else {
                    Self::from((*self.0).clone().$f(&*rhs.0))
                }
            }
        }
    };
}

proxy_op_int_opt!(Add, add);
proxy_op_int_opt!(Sub, sub);
proxy_op_int_opt!(Mul, mul);
proxy_op!(Div, div);
proxy_op!(Rem, rem);

impl Neg for Number {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(RefCount::new(-(*self.0).clone()))
    }
}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.is_real() && other.is_real() {
            self.0.re.partial_cmp(&other.0.re)
        } else if self.is_imaginary() && other.is_imaginary() {
            self.0.im.partial_cmp(&other.0.im)
        } else {
            None
        }
    }
}

impl Number {
    #[inline]
    pub fn rational<T>(num: T, den: T) -> Self
    where
        Self: From<T>,
    {
        Self::from(num) / Self::from(den)
    }

    #[inline]
    pub fn as_complex(&self) -> &Complex<BigRational> {
        &self.0
    }

    #[inline]
    pub fn is_real(&self) -> bool {
        self.0.im.is_zero()
    }

    #[inline]
    pub fn is_imaginary(&self) -> bool {
        self.0.re.is_zero() && !self.0.im.is_zero()
    }

    #[inline]
    pub fn as_real(&self) -> Option<BigRational> {
        if self.is_real() {
            Some(self.0.re.clone())
        } else {
            None
        }
    }

    #[inline]
    pub fn is_integer(&self) -> bool {
        self.is_real() && self.0.re.is_integer()
    }

    #[inline]
    pub fn is_uinteger(&self) -> bool {
        self.is_integer() && *self.0.re.numer() >= BigInt::zero()
    }

    #[inline]
    pub fn as_integer(&self) -> Option<BigInt> {
        if self.is_integer() {
            Some(self.0.re.numer().clone())
        } else {
            None
        }
    }

    #[inline]
    pub fn as_uinteger(&self) -> Option<BigUint> {
        if self.is_uinteger() {
            self.0.re.numer().to_biguint()
        } else {
            None
        }
    }

    #[inline]
    pub fn re(&self) -> Number {
        self.0.re.clone().into()
    }

    #[inline]
    pub fn im(&self) -> Number {
        self.0.im.clone().into()
    }

    pub(crate) fn pow(&self, other: &Self) -> Self {
        if self.is_real() && other.is_integer() {
            self.as_real()
                .unwrap()
                .pow(other.as_integer().unwrap())
                .into()
        } else {
            todo!()
        }
    }
}

macro_rules! from_integer {
    ($t:ty) => {
        impl From<$t> for Number {
            fn from(value: $t) -> Self {
                Self(RefCount::new(Complex::new(
                    BigRational::from(BigInt::from(value)),
                    Zero::zero(),
                )))
            }
        }
    };
}

from_integer!(usize);
from_integer!(u8);
from_integer!(u16);
from_integer!(u32);
from_integer!(u64);
from_integer!(u128);
from_integer!(isize);
from_integer!(i8);
from_integer!(i16);
from_integer!(i32);
from_integer!(i64);
from_integer!(i128);

impl From<f64> for Number {
    fn from(value: f64) -> Self {
        Self(RefCount::new(Complex::new(
            BigRational::from_f64(value).unwrap(),
            Zero::zero(),
        )))
    }
}

impl From<f32> for Number {
    fn from(value: f32) -> Self {
        Self(RefCount::new(Complex::new(
            BigRational::from_f32(value).unwrap(),
            Zero::zero(),
        )))
    }
}

impl From<Complex<BigRational>> for Number {
    fn from(value: Complex<BigRational>) -> Self {
        Self(RefCount::new(value))
    }
}

impl From<BigRational> for Number {
    fn from(value: BigRational) -> Self {
        Self(RefCount::new(Complex::new(value, Zero::zero())))
    }
}

impl From<BigInt> for Number {
    fn from(value: BigInt) -> Self {
        Self(RefCount::new(Complex::new(
            BigRational::from(value),
            Zero::zero(),
        )))
    }
}

impl From<BigUint> for Number {
    fn from(value: BigUint) -> Self {
        Self(RefCount::new(Complex::new(
            BigRational::from(BigInt::from(value)),
            Zero::zero(),
        )))
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.im.is_zero() {
            self.0.re.fmt(f)
        } else {
            self.0.fmt(f)
        }
    }
}

impl FromStr for Number {
    type Err = ParseComplexError<ParseRatioError>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(RefCount::new(s.parse()?)))
    }
}

impl From<Number> for Complex<BigRational> {
    fn from(value: Number) -> Self {
        (*value.0).clone()
    }
}

impl From<&Number> for Number {
    fn from(value: &Number) -> Self {
        value.clone()
    }
}

#[cfg(feature = "json")]
impl From<serde_json::Number> for Number {
    fn from(value: serde_json::Number) -> Self {
        value.as_str().parse().unwrap()
    }
}

#[cfg(feature = "sqlx")]
impl From<sqlx::types::BigDecimal> for Number {
    fn from(value: sqlx::types::BigDecimal) -> Self {
        let (bigint, exponent) = value.into_bigint_and_exponent();
        if exponent >= 0 {
            Self::from(
                BigRational::from(bigint)
                    * BigRational::new(BigInt::from(1), BigInt::from(10.pow(exponent as u32))),
            )
        } else {
            Self::from(bigint * 10.pow((-exponent) as u32))
        }
    }
}

impl TryFrom<Number> for BigRational {
    type Error = Number;

    fn try_from(value: Number) -> Result<Self, Self::Error> {
        if value.0.im.is_zero() {
            Ok(value.0.re.clone())
        } else {
            Err(value)
        }
    }
}

impl TryFrom<Number> for BigInt {
    type Error = Number;

    fn try_from(value: Number) -> Result<Self, Self::Error> {
        if value.0.im.is_zero() && value.0.re.denom().is_one() {
            Ok(value.0.re.numer().clone())
        } else {
            Err(value)
        }
    }
}

impl TryFrom<Number> for BigUint {
    type Error = Number;

    fn try_from(value: Number) -> Result<Self, Self::Error> {
        if value.0.im.is_zero() && value.0.re.denom().is_one() {
            value.0.re.numer().to_biguint().ok_or(value)
        } else {
            Err(value)
        }
    }
}

macro_rules! into_integer {
    (<$t:ty> via $f:ident) => {
        impl TryFrom<Number> for $t {
            type Error = Number;

            fn try_from(value: Number) -> Result<Self, Self::Error> {
                let Some(int) = value.as_integer() else {
                    return Err(value);
                };
                int.$f().ok_or(value)
            }
        }

        impl<'a> TryFrom<&'a Number> for $t {
            type Error = &'a Number;

            fn try_from(value: &'a Number) -> Result<Self, Self::Error> {
                let Some(int) = value.as_integer() else {
                    return Err(value);
                };
                int.$f().ok_or(value)
            }
        }
    };
}

into_integer!(<usize> via to_usize);
into_integer!(<u8> via to_u8);
into_integer!(<u16> via to_u16);
into_integer!(<u32> via to_u32);
into_integer!(<u64> via to_u64);
into_integer!(<u128> via to_u128);
into_integer!(<isize> via to_isize);
into_integer!(<i8> via to_i8);
into_integer!(<i16> via to_i16);
into_integer!(<i32> via to_i32);
into_integer!(<i64> via to_i64);
into_integer!(<i128> via to_i128);

impl<'a> TryFrom<&'a Number> for f64 {
    type Error = &'a Number;

    fn try_from(value: &'a Number) -> Result<Self, Self::Error> {
        let Some(rational) = value.as_real() else {
            return Err(value);
        };
        rational.to_f64().ok_or(value)
    }
}
