use crate::cactus::Cactus;
use crate::runtime::Value;
use crate::Instruction;

use super::Error;

#[derive(Clone, Debug, Default)]
pub(crate) struct Execution {
    pub cactus: Cactus<Value>,
    pub ip: usize,
}

impl Execution {
    pub fn new() -> Self {
        Self {
            cactus: Cactus::new(),
            ip: 0,
        }
    }

    pub fn branch(&mut self) -> Self {
        let branch = self.cactus.branch();
        Self {
            cactus: branch,
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

    pub fn read_offset(&mut self, instructions: &[u8]) -> Result<u32, Error> {
        let value = u32::from_le_bytes(
            instructions[self.ip..self.ip + 4]
                .try_into()
                .map_err(|_| Error::InternalRuntimeError)?,
        );
        self.ip += 4;
        Ok(value)
    }
}
