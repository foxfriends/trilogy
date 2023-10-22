use super::error::{ErrorKind, InternalRuntimeError};
use super::stack::InternalValue;
use super::{Error, Stack};
use crate::bytecode::chunk::Chunk;
use crate::callable::{Continuation, Procedure};
use crate::runtime::callable::{Callable, CallableKind};
use crate::{Offset, OpCode, Runtime, Value};

#[derive(Clone, Debug)]
pub(crate) struct Execution {
    pub ip: Offset,
    pub stack: Stack,
    stack_stack: Vec<(Offset, Stack)>,
}

impl Execution {
    pub fn new() -> Self {
        Self {
            ip: 0,
            stack: Stack::default(),
            stack_stack: vec![],
        }
    }

    pub fn branch(&mut self) -> Self {
        let branch = self.stack.branch();
        Self {
            stack: branch,
            stack_stack: vec![],
            ip: self.ip,
        }
    }

    pub fn read_opcode(&mut self, chunk: &Chunk) -> Result<OpCode, Error> {
        let instruction = chunk.bytes[self.ip as usize]
            .try_into()
            .map_err(|_| {
                InternalRuntimeError::InvalidOpcode(chunk.bytes[self.ip as usize], self.ip)
            })
            .map_err(|k| self.error(k))?;
        self.ip += 1;
        Ok(instruction)
    }

    pub fn read_offset(&mut self, chunk: &Chunk) -> Result<Offset, Error> {
        let value = u32::from_be_bytes(
            chunk.bytes[self.ip as usize..self.ip as usize + 4]
                .try_into()
                .unwrap(),
        );
        self.ip += 4;
        Ok(value)
    }

    pub fn read_constant(&mut self, chunk: &Chunk) -> Result<Value, Error> {
        let index = self.read_offset(chunk)?;
        chunk
            .constants
            .get(index as usize)
            .map(|value| value.structural_clone())
            .ok_or_else(|| self.error(InternalRuntimeError::MissingConstant))
    }

    pub fn current_continuation(&mut self) -> Continuation {
        Continuation::new(self.ip, self.stack.branch())
    }

    pub fn current_closure(&mut self) -> Procedure {
        Procedure::new_closure(self.ip, self.stack.branch())
    }

    pub fn call_continuation(
        &mut self,
        continuation: Continuation,
        args: Vec<InternalValue>,
    ) -> Result<(), Error> {
        let running_stack = continuation.stack();
        let paused_stack = std::mem::replace(&mut self.stack, running_stack);
        self.stack_stack.push((self.ip, paused_stack));
        self.stack.push_many(args);
        self.ip = continuation.ip();
        Ok(())
    }

    pub fn become_continuation(&mut self, continuation: Continuation, args: Vec<InternalValue>) {
        self.stack = continuation.stack();
        self.stack.push_many(args);
        self.ip = continuation.ip();
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

    fn call_procedure(&mut self, procedure: Procedure, args: Vec<InternalValue>) {
        self.stack.push_frame(self.ip, args, procedure.stack());
        self.ip = procedure.ip();
    }

    fn become_procedure(
        &mut self,
        procedure: Procedure,
        args: Vec<InternalValue>,
    ) -> Result<(), Error> {
        let ip = self.stack.pop_frame().map_err(|k| self.error(k))?;
        self.stack.push_frame(ip, args, procedure.stack());
        self.ip = procedure.ip();
        Ok(())
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

    pub fn is_set_local(&self, index: usize) -> Result<bool, Error> {
        self.stack.is_set_local(index).map_err(|k| self.error(k))
    }

    pub fn set_local(&mut self, index: usize, value: Value) -> Result<Option<Value>, Error> {
        self.stack
            .set_local(index, value)
            .map_err(|k| self.error(k))
    }

    pub fn push_unset(&mut self) {
        self.stack.push_unset();
    }

    pub fn unset_local(&mut self, index: usize) -> Result<Option<Value>, Error> {
        self.stack.unset_local(index).map_err(|k| self.error(k))
    }

    pub fn init_local(&mut self, index: usize, value: Value) -> Result<bool, Error> {
        self.stack
            .init_local(index, value)
            .map_err(|k| self.error(k))
    }

    pub fn stack_peek(&self) -> Result<Value, Error> {
        self.stack.at(0).map_err(|k| self.error(k))
    }

    pub fn stack_peek_raw(&self) -> Result<InternalValue, Error> {
        self.stack.at_raw(0).map_err(|k| self.error(k))
    }

    pub fn stack_pop(&mut self) -> Result<Value, Error> {
        self.stack_discard().and_then(|v| {
            v.ok_or_else(|| self.error(InternalRuntimeError::ExpectedValue("empty stack")))
        })
    }

    pub fn stack_discard(&mut self) -> Result<Option<Value>, Error> {
        self.stack.pop().map_err(|k| self.error(k))
    }

    pub fn stack_slide(&mut self, n: usize) -> Result<(), Error> {
        let top = self.stack_pop()?;
        let slide = self.stack.pop_n(n).map_err(|k| self.error(k))?;
        self.stack.push(top);
        self.stack.push_many(slide);
        Ok(())
    }

    pub fn stack_push(&mut self, value: Value) {
        self.stack.push(value);
    }

    pub fn stack_push_pointer(&mut self, pointer: usize) {
        self.stack.push_pointer(pointer);
    }

    pub fn call(&mut self, runtime: Runtime, arity: usize) -> Result<(), Error> {
        let arguments = self.stack.pop_n(arity).map_err(|k| self.error(k))?;
        let callable = self.stack.pop().map_err(|k| self.error(k))?;
        match callable {
            Some(Value::Callable(Callable(CallableKind::Continuation(continuation)))) => {
                self.call_continuation(continuation, arguments)?;
            }
            Some(Value::Callable(Callable(CallableKind::Procedure(procedure)))) => {
                self.call_procedure(procedure, arguments);
            }
            Some(Value::Callable(Callable(CallableKind::Native(native)))) => {
                let ret_val = native.call(
                    runtime,
                    arguments
                        .into_iter()
                        .map(|val| val.try_into_value())
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|k| self.error(k))?,
                );
                self.stack.push(ret_val);
            }
            _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
        }
        Ok(())
    }

    pub fn r#become(&mut self, runtime: Runtime, arity: usize) -> Result<(), Error> {
        let arguments = self.stack.pop_n(arity).map_err(|k| self.error(k))?;
        let callable = self.stack.pop().map_err(|k| self.error(k))?;
        match callable {
            Some(Value::Callable(Callable(CallableKind::Continuation(continuation)))) => {
                self.become_continuation(continuation, arguments);
            }
            Some(Value::Callable(Callable(CallableKind::Procedure(procedure)))) => {
                self.become_procedure(procedure, arguments)?;
            }
            Some(Value::Callable(Callable(CallableKind::Native(native)))) => {
                let ret_val = native.call(
                    runtime,
                    arguments
                        .into_iter()
                        .map(|val| val.try_into_value())
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|k| self.error(k))?,
                );
                self.r#return()?;
                self.stack.push(ret_val);
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
