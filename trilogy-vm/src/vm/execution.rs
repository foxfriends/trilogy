use super::{Error, Stack};
use crate::runtime::Continuation;
use crate::Instruction;

#[derive(Clone, Debug, Default)]
pub(crate) struct Execution {
    pub stack: Stack,
    pub ip: usize,
}

impl Execution {
    pub fn new() -> Self {
        Self {
            stack: Stack::default(),
            ip: 0,
        }
    }

    pub fn branch(&mut self) -> Self {
        let branch = self.stack.branch();
        Self {
            stack: branch,
            ip: self.ip,
        }
    }

    pub fn read_instruction(&mut self, instructions: &[u8]) -> Result<Instruction, Error> {
        let instruction = instructions[self.ip]
            .try_into()
            .map_err(|_| Error::InternalRuntimeError)?;
        self.ip += 1;
        Ok(instruction)
    }

    pub fn read_offset(&mut self, instructions: &[u8]) -> Result<usize, Error> {
        let value = usize::from_le_bytes(
            instructions[self.ip..self.ip + 4]
                .try_into()
                .map_err(|_| Error::InternalRuntimeError)?,
        );
        self.ip += 4;
        Ok(value)
    }

    pub fn current_continuation(&mut self) -> Continuation {
        Continuation::new(self.ip, self.stack.branch())
    }
}
