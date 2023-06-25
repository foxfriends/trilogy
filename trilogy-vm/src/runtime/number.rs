use num::{BigInt, BigRational, BigUint, Complex, Zero};
use std::ops::{Add, Div, Mul, Rem, Sub};

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

impl Number {
    pub fn is_integer(&self) -> bool {
        self.0.im == BigRational::zero() && self.0.re.is_integer()
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
