use bitvec::vec::BitVec;
use trilogy_parser::syntax;

#[derive(Clone, Debug)]
pub struct Bits(BitVec);

impl Bits {
    pub(super) fn convert(ast: syntax::BitsLiteral) -> Self {
        Self(ast.value())
    }

    pub fn value(&self) -> &BitVec {
        &self.0
    }
}

impl AsRef<BitVec> for Bits {
    fn as_ref(&self) -> &BitVec {
        &self.0
    }
}
