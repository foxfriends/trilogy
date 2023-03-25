use bitvec::vec::BitVec;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct BitsLiteral {
    span: Span,
    pub value: Bits,
}

#[derive(Clone, Debug)]
pub struct Bits(BitVec);
