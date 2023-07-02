use super::{Error, Stack};
use crate::bytecode::OpCode;
use crate::runtime::Continuation;

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

    pub fn read_opcode(&mut self, instructions: &[u8]) -> Result<OpCode, Error> {
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

    pub fn call_continuation(
        &mut self,
        continuation: Continuation,
        arity: usize,
    ) -> Result<(), Error> {
        let running_stack = continuation.stack();
        self.stack.continue_on(running_stack, arity)?;
        self.ip = continuation.ip();
        Ok(())
    }

    pub fn reset_continuation(&mut self) -> Result<(), Error> {
        if self.stack.return_to()? {
            let ip = self.stack.pop_pointer()?;
            self.ip = ip;
        }
        Ok(())
    }
}
