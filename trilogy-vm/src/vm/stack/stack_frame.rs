use super::{Cont, StackCell};
use crate::cactus::Slice;

#[derive(Clone)]
pub(crate) struct StackFrame<'a> {
    pub cactus: Option<Slice<'a, StackCell>>,
    pub cont: Cont,
}
