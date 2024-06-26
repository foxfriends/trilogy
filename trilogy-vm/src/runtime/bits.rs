use crate::Number;
use bitvec::prelude::*;
use num::bigint::Sign;
use num::BigInt;
use std::fmt::{self, Debug, Display};
use std::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr};
use std::sync::Arc;

/// A Trilogy Bits value.
///
/// Bits values are represented internally using types from the [`bitvec`][mod@bitvec] crate.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Bits(Arc<BitVec<usize, Msb0>>);

impl Bits {
    /// Creates a new empty bits value, containing 0 bits.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::Bits;
    /// let bits = Bits::new();
    /// assert_eq!(bits.len(), 0);
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a Bits value of the specified length, initialized to 0
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::Bits;
    /// let bits = Bits::new();
    /// assert_eq!(bits.len(), 0);
    /// ```
    #[inline]
    pub fn zeros(len: usize) -> Self {
        Self::from(bitvec![usize, Msb0; 0; len])
    }

    /// Returns the number of bits contained in this Bits value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::Bits;
    /// let bits = Bits::from(0u32);
    /// assert_eq!(bits.len(), 32);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if this Bits contains no bits.
    ///
    /// # Examples
    ///
    /// ```
    /// # use trilogy_vm::Bits;
    /// let bits = Bits::new();
    /// assert!(bits.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<bool> {
        self.0.get(index).as_deref().copied()
    }

    #[inline]
    pub fn to_bitvec(self) -> BitVec<usize, Msb0> {
        (*self.0).clone()
    }

    #[inline]
    pub fn concat(&self, other: &Self) -> Bits {
        let mut lhs = (*self.0).clone();
        let mut rhs = (*other.0).clone();
        lhs.append(&mut rhs);
        Bits::from(lhs)
    }

    #[inline]
    pub fn as_bitslice(&self) -> &BitSlice<usize, Msb0> {
        &self.0
    }
}

/// Converts this bits value to a number, interpreting it as an unsized big endian
/// integer in sign-magnitude representation (the most significant bit is the sign).
///
/// The empty bits value is interpreted as 0.
///
/// # Examples
///
/// ```
/// # use bitvec::prelude::*;
/// # use trilogy_vm::{Bits, Number};
/// //         0bb00000001 => 1
/// let b = Bits::from(bits![usize, Msb0; 0, 0, 0, 0, 0, 0, 0, 1]);
/// assert_eq!(Number::from(b), Number::from(1));
/// // 0bb0000000100000000 => 256
/// let b = Bits::from(bits![usize, Msb0; 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
/// assert_eq!(Number::from(b), Number::from(256));
/// //     0bb000100000001 => 257
/// let b = Bits::from(bits![usize, Msb0; 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1]);
/// assert_eq!(Number::from(b), Number::from(257));
///
/// //             0bb1010 => -(0bb010) => -2
/// let b = Bits::from(bits![usize, Msb0; 1, 0, 1, 0]);
/// assert_eq!(Number::from(b), Number::from(-2));
/// //             0bb1000 => -(0bb000) => 0
/// let b = Bits::from(bits![usize, Msb0; 1, 0, 0, 0]);
/// assert_eq!(Number::from(b), Number::from(0));
/// //             0bb1111 => -(0bb111) => -7
/// let b = Bits::from(bits![usize, Msb0; 1, 1, 1, 1]);
/// assert_eq!(Number::from(b), Number::from(-7));
/// //            0bb01111 => -(0bb1111) => -15
/// let b = Bits::from(bits![usize, Msb0; 0, 1, 1, 1, 1]);
/// assert_eq!(Number::from(b), Number::from(15));
///
/// //                 0bb => 0
/// let b = Bits::from(bits![usize, Msb0;]);
/// assert_eq!(Number::from(b), Number::from(0));
/// ```
impl From<Bits> for Number {
    fn from(value: Bits) -> Number {
        fn bit(slice: &BitSlice<usize, Msb0>, index: usize) -> u8 {
            if slice.get(index).as_deref().copied().unwrap_or(false) {
                0b00000001 << (slice.len() - 1 - index)
            } else {
                0
            }
        }

        if value.is_empty() {
            return Number::from(0);
        }

        let sign = if value.0[0] { Sign::Minus } else { Sign::Plus };
        let bytes = value.0[1..]
            .rchunks(8)
            .map(|slice| {
                u8::from_be(
                    bit(slice, 0)
                        | bit(slice, 1)
                        | bit(slice, 2)
                        | bit(slice, 3)
                        | bit(slice, 4)
                        | bit(slice, 5)
                        | bit(slice, 6)
                        | bit(slice, 7),
                )
            })
            .collect::<Vec<_>>();
        Number::from(BigInt::from_bytes_le(sign, &bytes))
    }
}

macro_rules! from_num {
    ($num:ty) => {
        impl From<$num> for Bits {
            fn from(value: $num) -> Self {
                <$num>::to_be_bytes(value).into_iter().collect()
            }
        }
    };
}

from_num!(u8);
from_num!(u16);
from_num!(u32);
from_num!(u64);
from_num!(u128);

impl IntoIterator for Bits {
    type Item = <BitVec<usize, Msb0> as IntoIterator>::Item;
    type IntoIter = <BitVec<usize, Msb0> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (*self.0).clone().into_iter()
    }
}

impl FromIterator<bool> for Bits {
    fn from_iter<T: IntoIterator<Item = bool>>(iter: T) -> Self {
        Self(Arc::new(BitVec::from_iter(iter)))
    }
}

impl From<Vec<bool>> for Bits {
    fn from(value: Vec<bool>) -> Self {
        value.into_iter().collect()
    }
}

impl From<&Bits> for Bits {
    fn from(value: &Bits) -> Self {
        value.clone()
    }
}

impl From<()> for Bits {
    fn from((): ()) -> Self {
        Self::new()
    }
}

impl From<BigInt> for Bits {
    fn from(value: BigInt) -> Self {
        let (sign, bits) = value.to_bytes_be();
        let mut sign = if sign == Sign::Minus {
            bitvec![usize, Msb0; 1]
        } else {
            bitvec![usize, Msb0; 0]
        };
        sign.extend(bits.into_iter().flat_map(|byte| {
            [
                byte & 0b10000000 > 0,
                byte & 0b01000000 > 0,
                byte & 0b00100000 > 0,
                byte & 0b00010000 > 0,
                byte & 0b00001000 > 0,
                byte & 0b00000100 > 0,
                byte & 0b00000010 > 0,
                byte & 0b00000001 > 0,
            ]
        }));
        Bits::from(sign)
    }
}

impl From<bool> for Bits {
    fn from(value: bool) -> Self {
        if value {
            Self::from(bits![usize, Msb0; 1])
        } else {
            Self::from(bits![usize, Msb0; 0])
        }
    }
}

impl From<&str> for Bits {
    fn from(value: &str) -> Self {
        value.bytes().collect()
    }
}

impl From<char> for Bits {
    fn from(value: char) -> Self {
        let mut bytes = [0; 4];
        let bytes = value.encode_utf8(&mut bytes);
        Bits::from(bytes as &str)
    }
}

impl FromIterator<u8> for Bits {
    fn from_iter<T: IntoIterator<Item = u8>>(value: T) -> Self {
        value
            .into_iter()
            .flat_map(|byte| {
                [
                    byte & 0b10000000 > 0,
                    byte & 0b01000000 > 0,
                    byte & 0b00100000 > 0,
                    byte & 0b00010000 > 0,
                    byte & 0b00001000 > 0,
                    byte & 0b00000100 > 0,
                    byte & 0b00000010 > 0,
                    byte & 0b00000001 > 0,
                ]
            })
            .collect()
    }
}

impl<'a> FromIterator<&'a u8> for Bits {
    fn from_iter<T: IntoIterator<Item = &'a u8>>(value: T) -> Self {
        value
            .into_iter()
            .flat_map(|byte| {
                [
                    byte & 0b10000000 > 0,
                    byte & 0b01000000 > 0,
                    byte & 0b00100000 > 0,
                    byte & 0b00010000 > 0,
                    byte & 0b00001000 > 0,
                    byte & 0b00000100 > 0,
                    byte & 0b00000010 > 0,
                    byte & 0b00000001 > 0,
                ]
            })
            .collect()
    }
}

impl From<BitVec<usize, Msb0>> for Bits {
    fn from(value: BitVec<usize, Msb0>) -> Self {
        Self(Arc::new(value))
    }
}

impl From<&BitSlice<usize, Msb0>> for Bits {
    fn from(value: &BitSlice<usize, Msb0>) -> Self {
        Self(Arc::new(value.to_bitvec()))
    }
}

impl From<Vec<u8>> for Bits {
    fn from(vec: Vec<u8>) -> Self {
        Self::from_iter(vec)
    }
}

#[cfg(feature = "sqlx")]
impl From<sqlx::types::BitVec> for Bits {
    fn from(value: sqlx::types::BitVec) -> Self {
        Self::from_iter(value.to_bytes())
    }
}

impl BitAnd for Bits {
    type Output = Bits;

    fn bitand(self, rhs: Self) -> Self::Output {
        let len = usize::max(self.0.len(), rhs.0.len());
        let lhs = (*self.0).clone();
        let mut rhs = (*rhs.0).clone();
        if lhs.len() > rhs.len() {
            rhs.extend(BitVec::<usize, Lsb0>::repeat(false, len - rhs.len()));
        } else {
            rhs.truncate(lhs.len());
        }
        Self::from(lhs & rhs)
    }
}

impl BitOr for Bits {
    type Output = Bits;

    fn bitor(self, rhs: Self) -> Self::Output {
        let len = usize::max(self.0.len(), rhs.0.len());
        let lhs = (*self.0).clone();
        let mut rhs = (*rhs.0).clone();
        if lhs.len() > rhs.len() {
            rhs.extend(BitVec::<usize, Lsb0>::repeat(false, len - rhs.len()));
        } else {
            rhs.truncate(lhs.len());
        }
        Self::from(lhs | rhs)
    }
}

impl BitXor for Bits {
    type Output = Bits;

    fn bitxor(self, rhs: Self) -> Self::Output {
        let len = usize::max(self.0.len(), rhs.0.len());
        let lhs = (*self.0).clone();
        let mut rhs = (*rhs.0).clone();
        if lhs.len() > rhs.len() {
            rhs.extend(BitVec::<usize, Lsb0>::repeat(false, len - rhs.len()));
        } else {
            rhs.truncate(lhs.len());
        }
        Self::from(lhs ^ rhs)
    }
}

impl Shl<usize> for Bits {
    type Output = Bits;

    fn shl(self, rhs: usize) -> Self::Output {
        let len = usize::min(self.0.len(), rhs);
        let mut val = (*self.0).clone();
        val.shift_left(len);
        Self::from(val)
    }
}

impl Shr<usize> for Bits {
    type Output = Bits;

    fn shr(self, rhs: usize) -> Self::Output {
        let len = usize::min(self.0.len(), rhs);
        let mut val = (*self.0).clone();
        val.shift_right(len);
        Self::from(val)
    }
}

impl Not for Bits {
    type Output = Bits;

    fn not(self) -> Self::Output {
        Self::from(!(*self.0).clone())
    }
}

impl Display for Bits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for bit in self.0.as_ref() {
            write!(f, "{}", if *bit { 1 } else { 0 })?;
        }
        Ok(())
    }
}

impl Debug for Bits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0bb")?;
        for bit in self.0.as_ref() {
            write!(f, "{}", if *bit { 1 } else { 0 })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bitvec::bitvec;

    #[test]
    fn bitand() {
        let lhs = Bits::from(bitvec![usize, Msb0; 0, 1, 0, 1]);
        let rhs = Bits::from(bitvec![usize, Msb0; 0, 0, 1, 1]);
        assert_eq!(lhs & rhs, Bits::from(bitvec![usize, Msb0; 0, 0, 0, 1]));
    }

    #[test]
    fn bitand_rhs_extend() {
        let lhs = Bits::from(bitvec![usize, Msb0; 1, 1]);
        let rhs = Bits::from(bitvec![usize, Msb0; 1]);
        assert_eq!(lhs & rhs, Bits::from(bitvec![usize, Msb0; 1, 0]));
    }

    #[test]
    fn bitand_rhs_trunc() {
        let rhs = Bits::from(bitvec![usize, Msb0; 1, 1]);
        let lhs = Bits::from(bitvec![usize, Msb0; 1]);
        assert_eq!(lhs & rhs, Bits::from(bitvec![usize, Msb0; 1]));
    }

    #[test]
    fn bitor() {
        let lhs = Bits::from(bitvec![usize, Msb0; 0, 1, 0, 1]);
        let rhs = Bits::from(bitvec![usize, Msb0; 0, 0, 1, 1]);
        assert_eq!(lhs | rhs, Bits::from(bitvec![usize, Msb0; 0, 1, 1, 1]));
    }

    #[test]
    fn bitor_rhs_extend() {
        let lhs = Bits::from(bitvec![usize, Msb0; 0, 0]);
        let rhs = Bits::from(bitvec![usize, Msb0; 1]);
        assert_eq!(lhs | rhs, Bits::from(bitvec![usize, Msb0; 1, 0]));
    }

    #[test]
    fn bitor_rhs_trunc() {
        let rhs = Bits::from(bitvec![usize, Msb0; 0, 0]);
        let lhs = Bits::from(bitvec![usize, Msb0; 1]);
        assert_eq!(lhs | rhs, Bits::from(bitvec![usize, Msb0; 1]));
    }

    #[test]
    fn bitxor() {
        let lhs = Bits::from(bitvec![usize, Msb0; 0, 1, 0, 1]);
        let rhs = Bits::from(bitvec![usize, Msb0; 0, 0, 1, 1]);
        assert_eq!(lhs ^ rhs, Bits::from(bitvec![usize, Msb0; 0, 1, 1, 0]));
    }

    #[test]
    fn bitxor_rhs_extend() {
        let lhs = Bits::from(bitvec![usize, Msb0; 0, 0]);
        let rhs = Bits::from(bitvec![usize, Msb0; 1]);
        assert_eq!(lhs ^ rhs, Bits::from(bitvec![usize, Msb0; 1, 0]));
    }

    #[test]
    fn bitxor_rhs_trunc() {
        let rhs = Bits::from(bitvec![usize, Msb0; 0, 0]);
        let lhs = Bits::from(bitvec![usize, Msb0; 1]);
        assert_eq!(lhs ^ rhs, Bits::from(bitvec![usize, Msb0; 1]));
    }

    #[test]
    fn not() {
        let val = Bits::from(bitvec![usize, Msb0; 0, 1]);
        assert_eq!(!val, Bits::from(bitvec![usize, Msb0; 1, 0]));
    }

    #[test]
    fn shl() {
        let val = Bits::from(bitvec![usize, Msb0; 0, 0, 1, 0]);
        assert_eq!(val << 2, Bits::from(bitvec![usize, Msb0; 1, 0, 0, 0]))
    }

    #[test]
    fn shr() {
        let val = Bits::from(bitvec![usize, Msb0; 1, 1, 0]);
        assert_eq!(val >> 2, Bits::from(bitvec![usize, Msb0; 0, 0, 1]))
    }

    #[test]
    fn ord_gt() {
        let lhs = Bits::from(bitvec![usize, Msb0; 1, 0]);
        let rhs = Bits::from(bitvec![usize, Msb0; 0, 1]);
        assert!(lhs > rhs)
    }

    #[test]
    fn ord_lt() {
        let lhs = Bits::from(bitvec![usize, Msb0; 0, 1]);
        let rhs = Bits::from(bitvec![usize, Msb0; 1, 0]);
        assert!(lhs < rhs)
    }

    #[test]
    fn display() {
        let bits = Bits::from(bitvec![usize, Msb0; 0, 1, 0, 0, 1]);
        assert_eq!(format!("{bits}"), "01001");
    }

    #[test]
    fn debug() {
        let bits = Bits::from(bitvec![usize, Msb0; 0, 1, 0, 0, 1]);
        assert_eq!(format!("{bits:?}"), "0bb01001");
    }
}
