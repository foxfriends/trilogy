use crate::runtime::Value;
use crate::{Offset, OpCode};

#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) enum Parameter {
    Value(Value),
    Label(String),
    Offset(u32),
    Reference(String),
}

pub(crate) struct Line {
    pub labels: Vec<String>,
    pub opcode: OpCode,
    pub value: Option<Parameter>,
}

impl Line {
    pub(crate) fn byte_len(&self) -> Offset {
        if self.value.is_some() {
            5
        } else {
            1
        }
    }
}
