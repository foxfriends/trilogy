use crate::Number;
use bitvec::order::Lsb0;
use bitvec::slice::BitSlice;
use bitvec::vec::BitVec;
use num::BigUint;
use std::fmt::{self, Debug, Display};
use std::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr};

/// A Trilogy Bits value.
///
/// Bits values are represented internally using types from the [`bitvec`][] crate.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Bits(BitVec);

impl Bits {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<bool> {
        self.0.get(index).as_deref().copied()
    }

    pub fn set(&mut self, index: usize, value: bool) {
        self.0.set(index, value);
    }

    pub fn to_bitvec(self) -> BitVec {
        self.0
    }

    pub fn as_bitslice(&self) -> &BitSlice {
        &self.0
    }
}

/// Converts this bits value to a number, interpreting it as an unsized integer.
///
/// # Examples
///
/// ```
/// # use bitvec::prelude::*;
/// # use trilogy_vm::{Bits, Number};
/// //         0bb00000001 => 1
/// let b = Bits::from(bits![0, 0, 0, 0, 0, 0, 0, 1]);
/// assert_eq!(Number::from(b), Number::from(1));
/// // 0bb0000000100000000 => 256
/// let b = Bits::from(bits![0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
/// assert_eq!(Number::from(b), Number::from(256));
/// //     0bb000100000001 => 257
/// let b = Bits::from(bits![0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1]);
/// assert_eq!(Number::from(b), Number::from(257));
/// ```
impl From<Bits> for Number {
    fn from(value: Bits) -> Number {
        fn bit(slice: &BitSlice, index: usize) -> u8 {
            if slice.get(index).as_deref().copied().unwrap_or(false) {
                0b00000001 << (slice.len() - 1 - index)
            } else {
                0
            }
        }

        let bytes = value
            .0
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
        Number::from(BigUint::from_bytes_le(&bytes))
    }
}

impl IntoIterator for Bits {
    type Item = <BitVec as IntoIterator>::Item;
    type IntoIter = <BitVec as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<bool> for Bits {
    fn from_iter<T: IntoIterator<Item = bool>>(iter: T) -> Self {
        Self(BitVec::from_iter(iter))
    }
}

impl From<Vec<bool>> for Bits {
    fn from(value: Vec<bool>) -> Self {
        value.into_iter().collect()
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

impl From<BitVec> for Bits {
    fn from(value: BitVec) -> Self {
        Self(value)
    }
}

impl From<&BitSlice> for Bits {
    fn from(value: &BitSlice) -> Self {
        Self(value.to_bitvec())
    }
}

impl BitAnd for Bits {
    type Output = Bits;

    fn bitand(self, rhs: Self) -> Self::Output {
        let len = usize::max(self.0.len(), rhs.0.len());
        let mut lhs_ext = self.0.clone();
        lhs_ext.extend(BitVec::<usize, Lsb0>::repeat(false, len - self.0.len()));
        let mut rhs_ext = rhs.0.clone();
        rhs_ext.extend(BitVec::<usize, Lsb0>::repeat(false, len - rhs.0.len()));
        Self(lhs_ext & rhs_ext)
    }
}

impl BitOr for Bits {
    type Output = Bits;

    fn bitor(self, rhs: Self) -> Self::Output {
        let len = usize::max(self.0.len(), rhs.0.len());
        let mut lhs_ext = self.0.clone();
        lhs_ext.extend(BitVec::<usize, Lsb0>::repeat(false, len - self.0.len()));
        let mut rhs_ext = rhs.0.clone();
        rhs_ext.extend(BitVec::<usize, Lsb0>::repeat(false, len - rhs.0.len()));
        Self(lhs_ext | rhs_ext)
    }
}

impl BitXor for Bits {
    type Output = Bits;

    fn bitxor(self, rhs: Self) -> Self::Output {
        let len = usize::max(self.0.len(), rhs.0.len());
        let mut lhs_ext = self.0.clone();
        lhs_ext.extend(BitVec::<usize, Lsb0>::repeat(false, len - self.0.len()));
        let mut rhs_ext = rhs.0.clone();
        rhs_ext.extend(BitVec::<usize, Lsb0>::repeat(false, len - rhs.0.len()));
        Self(lhs_ext ^ rhs_ext)
    }
}

impl Shl<usize> for Bits {
    type Output = Bits;

    fn shl(mut self, rhs: usize) -> Self::Output {
        let len = usize::min(self.0.len(), rhs);
        self.0.shift_left(len);
        self
    }
}

impl Shr<usize> for Bits {
    type Output = Bits;

    fn shr(mut self, rhs: usize) -> Self::Output {
        let len = usize::min(self.0.len(), rhs);
        self.0.shift_right(len);
        self
    }
}

impl Not for Bits {
    type Output = Bits;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl Display for Bits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for bit in &self.0 {
            write!(f, "{}", if *bit { 1 } else { 0 })?;
        }
        Ok(())
    }
}

impl Debug for Bits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0bb")?;
        for bit in &self.0 {
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
        let lhs = Bits(bitvec![0, 1, 0, 1]);
        let rhs = Bits(bitvec![0, 0, 1, 1]);
        assert_eq!(lhs & rhs, Bits(bitvec![0, 0, 0, 1]));
    }

    #[test]
    fn bitand_rhs_extend() {
        let lhs = Bits(bitvec![1, 1]);
        let rhs = Bits(bitvec![1]);
        assert_eq!(lhs & rhs, Bits(bitvec![1, 0]));
    }

    #[test]
    fn bitand_lhs_extend() {
        let rhs = Bits(bitvec![1, 1]);
        let lhs = Bits(bitvec![1]);
        assert_eq!(lhs & rhs, Bits(bitvec![1, 0]));
    }

    #[test]
    fn bitor() {
        let lhs = Bits(bitvec![0, 1, 0, 1]);
        let rhs = Bits(bitvec![0, 0, 1, 1]);
        assert_eq!(lhs | rhs, Bits(bitvec![0, 1, 1, 1]));
    }

    #[test]
    fn bitor_rhs_extend() {
        let lhs = Bits(bitvec![0, 0]);
        let rhs = Bits(bitvec![1]);
        assert_eq!(lhs | rhs, Bits(bitvec![1, 0]));
    }

    #[test]
    fn bitor_lhs_extend() {
        let rhs = Bits(bitvec![0, 0]);
        let lhs = Bits(bitvec![1]);
        assert_eq!(lhs | rhs, Bits(bitvec![1, 0]));
    }

    #[test]
    fn bitxor() {
        let lhs = Bits(bitvec![0, 1, 0, 1]);
        let rhs = Bits(bitvec![0, 0, 1, 1]);
        assert_eq!(lhs ^ rhs, Bits(bitvec![0, 1, 1, 0]));
    }

    #[test]
    fn bitxor_rhs_extend() {
        let lhs = Bits(bitvec![0, 0]);
        let rhs = Bits(bitvec![1]);
        assert_eq!(lhs ^ rhs, Bits(bitvec![1, 0]));
    }

    #[test]
    fn bitxor_lhs_extend() {
        let rhs = Bits(bitvec![0, 0]);
        let lhs = Bits(bitvec![1]);
        assert_eq!(lhs ^ rhs, Bits(bitvec![1, 0]));
    }

    #[test]
    fn not() {
        let val = Bits(bitvec![0, 1]);
        assert_eq!(!val, Bits(bitvec![1, 0]));
    }

    #[test]
    fn shl() {
        let val = Bits(bitvec![0, 0, 1, 0]);
        assert_eq!(val << 2, Bits(bitvec![1, 0, 0, 0]))
    }

    #[test]
    fn shr() {
        let val = Bits(bitvec![1, 1, 0]);
        assert_eq!(val >> 2, Bits(bitvec![0, 0, 1]))
    }

    #[test]
    fn ord_gt() {
        let lhs = Bits(bitvec![1, 0]);
        let rhs = Bits(bitvec![0, 1]);
        assert!(lhs > rhs)
    }

    #[test]
    fn ord_lt() {
        let lhs = Bits(bitvec![0, 1]);
        let rhs = Bits(bitvec![1, 0]);
        assert!(lhs < rhs)
    }

    #[test]
    fn display() {
        let bits = Bits(bitvec![0, 1, 0, 0, 1]);
        assert_eq!(format!("{bits}"), "01001");
    }

    #[test]
    fn debug() {
        let bits = Bits(bitvec![0, 1, 0, 0, 1]);
        assert_eq!(format!("{bits:?}"), "0bb01001");
    }
}
