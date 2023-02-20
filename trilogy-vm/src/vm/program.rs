use crate::bytecode::Instruction;

#[derive(Clone, Debug)]
pub struct Program {
    instructions: Vec<Instruction>,
}
