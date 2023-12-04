use crate::runtime::Value;
use crate::OpCode;

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
