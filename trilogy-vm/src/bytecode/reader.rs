use crate::Instruction;

pub trait Reader {
    type Error;

    fn read_instruction(&mut self) -> Result<Instruction, Self::Error>;
    fn jump(&mut self, offset: usize);
    fn jump_back(&mut self, offset: usize);
    fn seek(&mut self, ip: usize);
}
