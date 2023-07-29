use super::error::{ErrorKind, InternalRuntimeError};
use super::{Error, Stack};
use crate::bytecode::OpCode;
use crate::runtime::{Continuation, Procedure};
use crate::Value;

#[derive(Clone, Debug, Default)]
pub(crate) struct Execution {
    pub ip: usize,
    stack: Stack,
    stack_stack: Vec<(usize, Stack)>,
}

impl Execution {
    pub fn branch(&mut self) -> Self {
        let branch = self.stack.branch();
        Self {
            stack: branch,
            stack_stack: vec![],
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
        args: Vec<Value>,
    ) -> Result<(), Error> {
        let running_stack = continuation.stack();
        let paused_stack = std::mem::replace(&mut self.stack, running_stack);
        self.stack_stack.push((self.ip, paused_stack));
        self.stack.push_many(args);
        self.ip = continuation.ip();
        Ok(())
    }

    pub fn reset_continuation(&mut self) -> Result<(), Error> {
        let (ip, running_stack) = self.stack_stack.pop().ok_or_else(|| {
            self.error(ErrorKind::InternalRuntimeError(
                InternalRuntimeError::ExpectedStack,
            ))
        })?;
        self.ip = ip;
        self.stack = running_stack;
        Ok(())
    }

    fn call_procedure(&mut self, procedure: Procedure, args: Vec<Value>) {
        self.stack.push_frame(self.ip, args);
        self.ip = procedure.ip();
    }

    pub fn error<K>(&self, kind: K) -> Error
    where
        ErrorKind: From<K>,
    {
        Error {
            ip: self.ip,
            stack_dump: self.stack.clone(),
            kind: kind.into(),
        }
    }

    pub fn read_local(&self, index: usize) -> Result<Value, Error> {
        self.stack.at_local(index).map_err(|k| self.error(k))
    }

    pub fn set_local(&mut self, index: usize, value: Value) -> Result<Value, Error> {
        self.stack
            .replace_at_local(index, value)
            .map_err(|k| self.error(k))
    }

    pub fn read_register(&self, index: usize) -> Result<Value, Error> {
        self.stack.at(index).map_err(|k| self.error(k))
    }

    pub fn set_register(&mut self, index: usize, value: Value) -> Result<Value, Error> {
        self.stack
            .replace_at(index, value)
            .map_err(|k| self.error(k))
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

    pub fn call(&mut self, arity: usize) -> Result<(), Error> {
        let arguments = self.stack.pop_n(arity).map_err(|k| self.error(k))?;
        let callable = self.stack.pop().map_err(|k| self.error(k))?;
        match callable {
            Value::Continuation(continuation) => {
                self.call_continuation(continuation, arguments)?;
            }
            Value::Procedure(procedure) => {
                self.call_procedure(procedure, arguments);
            }
            _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
        }
        Ok(())
    }

    pub fn stack_pop_pointer(&mut self) -> Result<usize, Error> {
        self.stack.pop_pointer().map_err(|k| self.error(k))
    }

    pub fn r#return(&mut self) -> Result<(), Error> {
        let ip = self.stack.pop_frame().map_err(|k| self.error(k))?;
        self.ip = ip;
        Ok(())
    }
}
