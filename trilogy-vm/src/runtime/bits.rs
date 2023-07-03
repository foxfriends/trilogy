use bitvec::order::Lsb0;
use bitvec::vec::BitVec;
use std::fmt::{self, Display};
use std::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Bits(BitVec);

impl Bits {
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
}

impl FromIterator<bool> for Bits {
    fn from_iter<T: IntoIterator<Item = bool>>(iter: T) -> Self {
        Self(BitVec::from_iter(iter))
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
        self.0.extend(BitVec::<usize, Lsb0>::repeat(false, rhs));
        self
    }
}

impl Shr<usize> for Bits {
    type Output = Bits;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn shr(self, rhs: usize) -> Self::Output {
        Self(self.0[..self.0.len() - rhs].to_bitvec())
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
        write!(f, "0b")?;
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
        let val = Bits(bitvec![1, 0]);
        assert_eq!(val << 2, Bits(bitvec![1, 0, 0, 0]))
    }

    #[test]
    fn shr() {
        let val = Bits(bitvec![1, 1, 0]);
        assert_eq!(val >> 2, Bits(bitvec![1]))
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
        assert_eq!(format!("{bits}"), "0b01001");
    }
}
