use super::{Cont, Offset, StackCell};
use crate::cactus::Slice;

#[derive(Clone, Debug)]
pub(crate) struct StackFrame<'a> {
    /// The stack values held by this frame.
    pub slice: Option<Slice<'a, StackCell>>,
    /// The IP (or callback) from which this frame was entered, and to be returned to when finishe.
    pub cont: Cont,
    /// The frame pointer
    pub fp: usize,
    /// The IP at which the next stack frame was entered
    pub here: Option<Offset>,
}
