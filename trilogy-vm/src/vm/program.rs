use crate::bytecode::Instruction;
use crate::runtime::Value;

#[derive(Clone, Debug)]
pub struct Program {
    pub(crate) constants: Vec<Value>,
    pub(crate) instructions: Vec<Instruction>,
}
