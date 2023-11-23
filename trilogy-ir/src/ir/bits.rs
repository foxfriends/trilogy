use bitvec::prelude::*;
use trilogy_parser::syntax;

#[derive(Clone, Debug)]
pub struct Bits(BitVec<usize, Msb0>);

impl Bits {
    pub(super) fn convert(ast: syntax::BitsLiteral) -> Self {
        Self(ast.value())
    }

    pub fn value(&self) -> &BitVec<usize, Msb0> {
        &self.0
    }
}

impl AsRef<BitVec<usize, Msb0>> for Bits {
    fn as_ref(&self) -> &BitVec<usize, Msb0> {
        &self.0
    }
}
