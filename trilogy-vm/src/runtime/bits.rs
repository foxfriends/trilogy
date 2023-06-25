use bitvec::vec::BitVec;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Bits(BitVec);

impl Bits {
    pub fn get(&self, index: usize) -> Option<bool> {
        self.0.get(index).as_deref().copied()
    }

    pub fn set(&mut self, index: usize, value: bool) {
        self.0.set(index, value);
    }
}
