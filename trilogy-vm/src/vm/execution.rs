use super::error::{ErrorKind, InternalRuntimeError};
use super::{Error, Stack};
use crate::bytecode::OpCode;
use crate::runtime::Continuation;
use crate::Value;

#[derive(Clone, Debug, Default)]
pub(crate) struct Execution {
    stack: Stack,
    pub ip: usize,
}

impl Execution {
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
            .map_err(|_| InternalRuntimeError::InvalidOpcode)
            .map_err(|k| self.error(k))?;
        self.ip += 1;
        Ok(instruction)
    }

    pub fn read_offset(&mut self, instructions: &[u8]) -> Result<usize, Error> {
        let value = u32::from_be_bytes(
            instructions[self.ip..self.ip + 4]
                .try_into()
                .map_err(|_| InternalRuntimeError::InvalidOffset)
                .map_err(|k| self.error(k))?,
        );
        self.ip += 4;
        Ok(value as usize)
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
        self.stack
            .continue_on(running_stack, arity)
            .map_err(|k| self.error(k))?;
        self.ip = continuation.ip();
        Ok(())
    }

    pub fn reset_continuation(&mut self) -> Result<(), Error> {
        if self.stack.return_to().map_err(|k| self.error(k))? {
            let ip = self.stack.pop_return().map_err(|k| self.error(k))?;
            self.ip = ip;
        }
        Ok(())
    }

    pub fn error<K>(&self, kind: K) -> Error
    where
        ErrorKind: From<K>,
    {
        Error {
            ip: self.ip,
            kind: kind.into(),
        }
    }

    pub fn stack_at(&self, index: usize) -> Result<Value, Error> {
        self.stack.at(index).map_err(|k| self.error(k))
    }

    pub fn stack_pop(&mut self) -> Result<Value, Error> {
        self.stack.pop().map_err(|k| self.error(k))
    }

    pub fn stack_push(&mut self, value: Value) {
        self.stack.push(value);
    }

    pub fn stack_push_pointer(&mut self, pointer: usize) {
        self.stack.push_pointer(pointer);
    }

    pub fn stack_replace_at(&mut self, index: usize, value: Value) -> Result<Value, Error> {
        self.stack
            .replace_at(index, value)
            .map_err(|k| self.error(k))
    }

    pub fn stack_replace_with_return(
        &mut self,
        index: usize,
        pointer: usize,
    ) -> Result<Value, Error> {
        self.stack
            .replace_with_return(index, pointer)
            .map_err(|k| self.error(k))
    }

    pub fn stack_pop_pointer(&mut self) -> Result<usize, Error> {
        self.stack.pop_pointer().map_err(|k| self.error(k))
    }

    pub fn stack_pop_return(&mut self) -> Result<usize, Error> {
        self.stack.pop_return().map_err(|k| self.error(k))
    }
}
