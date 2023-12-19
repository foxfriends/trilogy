use super::{Cont, StackCell};
use crate::cactus::Slice;

#[derive(Clone, Debug)]
pub(crate) struct StackFrame<'a> {
    pub cactus: Option<Slice<'a, StackCell>>,
    pub cont: Cont,
    pub fp: usize,
}
