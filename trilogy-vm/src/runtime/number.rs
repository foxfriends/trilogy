use num::{BigInt, BigRational, BigUint, Complex, Zero};
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Number(Complex<BigRational>);

impl Add for Number {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Number {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul for Number {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Div for Number {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl Rem for Number {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0)
    }
}

impl Neg for Number {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
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
    pub fn as_complex(&self) -> Complex<BigRational> {
        self.0.clone()
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
}

impl<T> From<T> for Number
where
    Complex<BigRational>: From<T>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}
