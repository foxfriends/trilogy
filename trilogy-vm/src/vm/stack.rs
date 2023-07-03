use super::error::InternalRuntimeError;
use crate::{cactus::Cactus, Value};

#[derive(Clone, Debug)]
enum InternalValue {
    Value(Value),
    Pointer(usize),
    Stack(Stack),
}

impl InternalValue {
    fn try_into_value(self) -> Result<Value, InternalRuntimeError> {
        match self {
            InternalValue::Value(value) => Ok(value),
            _ => Err(InternalRuntimeError::ExpectedValue),
        }
    }

    fn try_into_pointer(self) -> Result<usize, InternalRuntimeError> {
        match self {
            InternalValue::Pointer(pointer) => Ok(pointer),
            _ => Err(InternalRuntimeError::ExpectedPointer),
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct Stack(Cactus<InternalValue>);

impl Stack {
    pub(crate) fn branch(&mut self) -> Self {
        Self(self.0.branch())
    }

    pub(crate) fn push(&mut self, value: Value) {
        self.0.push(InternalValue::Value(value));
    }

    pub(crate) fn pop(&mut self) -> Result<Value, InternalRuntimeError> {
        self.0
            .pop()
            .ok_or(InternalRuntimeError::ExpectedValue)
            .and_then(InternalValue::try_into_value)
    }

    pub(crate) fn at(&self, index: usize) -> Result<Value, InternalRuntimeError> {
        self.0
            .at(index)
            .ok_or(InternalRuntimeError::ExpectedValue)
            .and_then(InternalValue::try_into_value)
    }

    pub(crate) fn pop_pointer(&mut self) -> Result<usize, InternalRuntimeError> {
        self.0
            .pop()
            .ok_or(InternalRuntimeError::ExpectedPointer)
            .and_then(InternalValue::try_into_pointer)
    }

    pub(crate) fn replace_with_pointer(
        &mut self,
        index: usize,
        pointer: usize,
    ) -> Result<Value, InternalRuntimeError> {
        self.0
            .replace_at(index, InternalValue::Pointer(pointer))
            .map_err(|_| InternalRuntimeError::ExpectedValue)
            .and_then(InternalValue::try_into_value)
    }

    pub(crate) fn replace_at(
        &mut self,
        index: usize,
        value: Value,
    ) -> Result<Value, InternalRuntimeError> {
        self.0
            .replace_at(index, InternalValue::Value(value))
            .map_err(|_| InternalRuntimeError::ExpectedValue)
            .and_then(InternalValue::try_into_value)
    }

    pub(crate) fn continue_on(
        &mut self,
        stack: Stack,
        offset: usize,
    ) -> Result<(), InternalRuntimeError> {
        // NOTE: it would be best if the transfer were performed at the VirtualMachine
        // level, but because of privacy it's just more convenient to do it here. Move
        // it later if it ever comes up.
        let transfer = self
            .0
            .detach_at(offset)
            .ok_or(InternalRuntimeError::ExpectedValue)?;
        let return_to = std::mem::replace(self, stack);
        self.0.push(InternalValue::Stack(return_to));
        self.0.attach(transfer);
        Ok(())
    }

    pub(crate) fn return_to(&mut self) -> Result<bool, InternalRuntimeError> {
        let value = self.0.pop().ok_or(InternalRuntimeError::ExpectedStack)?;
        if let InternalValue::Stack(stack) = value {
            // NOTE: this seems to be "correct" but... a bit disappointing in that we have no
            // way of detecting that this was actually not just improper stack usage. Fortunately,
            // programs should still end up being invalid as we will soon try to pop a value from
            // the stack to find that we are on the wrong stack and a stack was popped instead,
            // causing an internal runtime error still, just not at the optimal moment.
            *self = stack;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
