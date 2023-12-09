use num::complex::ParseComplexError;
use num::rational::ParseRatioError;
use num::traits::Pow;
use num::{BigInt, BigRational, BigUint, Complex, One, ToPrimitive, Zero};
use std::fmt::{self, Display};
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};
use std::str::FromStr;
use std::sync::Arc;

/// A Trilogy Number value.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Number(Arc<Complex<BigRational>>);

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

proxy_op!(Add, add);
proxy_op!(Sub, sub);
proxy_op!(Mul, mul);
proxy_op!(Div, div);
proxy_op!(Rem, rem);

impl Neg for Number {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(Arc::new(-(*self.0).clone()))
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
    pub fn rational<T>(num: T, den: T) -> Self
    where
        Self: From<T>,
    {
        Self::from(num) / Self::from(den)
    }

    pub fn as_complex(&self) -> &Complex<BigRational> {
        &self.0
    }

    pub fn is_real(&self) -> bool {
        self.0.im.is_zero()
    }

    pub fn is_imaginary(&self) -> bool {
        self.0.re.is_zero() && !self.0.im.is_zero()
    }

    pub fn as_real(&self) -> Option<BigRational> {
        if self.is_real() {
            Some(self.0.re.clone())
        } else {
            None
        }
    }

    pub fn is_integer(&self) -> bool {
        self.is_real() && self.0.re.is_integer()
    }

    pub fn is_uinteger(&self) -> bool {
        self.is_integer() && *self.0.re.numer() >= BigInt::zero()
    }

    pub fn as_integer(&self) -> Option<BigInt> {
        if self.is_integer() {
            Some(self.0.re.numer().clone())
        } else {
            None
        }
    }

    pub fn as_uinteger(&self) -> Option<BigUint> {
        if self.is_uinteger() {
            self.0.re.numer().to_biguint()
        } else {
            None
        }
    }

    pub fn re(&self) -> Number {
        self.0.re.clone().into()
    }

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
                Self(Arc::new(Complex::new(
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

impl From<Complex<BigRational>> for Number {
    fn from(value: Complex<BigRational>) -> Self {
        Self(Arc::new(value))
    }
}

impl From<BigRational> for Number {
    fn from(value: BigRational) -> Self {
        Self(Arc::new(Complex::new(value, Zero::zero())))
    }
}

impl From<BigInt> for Number {
    fn from(value: BigInt) -> Self {
        Self(Arc::new(Complex::new(
            BigRational::from(value),
            Zero::zero(),
        )))
    }
}

impl From<BigUint> for Number {
    fn from(value: BigUint) -> Self {
        Self(Arc::new(Complex::new(
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
        Ok(Self(Arc::new(s.parse()?)))
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
