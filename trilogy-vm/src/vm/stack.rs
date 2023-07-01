use super::Error;
use crate::{cactus::Cactus, Value};

#[derive(Clone, Debug)]
enum InternalValue {
    Value(Value),
    Pointer(usize),
}

impl InternalValue {
    fn try_into_value(self) -> Result<Value, Error> {
        match self {
            InternalValue::Value(value) => Ok(value),
            InternalValue::Pointer(..) => Err(Error::InternalRuntimeError),
        }
    }

    fn try_into_pointer(self) -> Result<usize, Error> {
        match self {
            InternalValue::Pointer(pointer) => Ok(pointer),
            InternalValue::Value(..) => Err(Error::InternalRuntimeError),
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct Stack(Cactus<InternalValue>);

impl Stack {
    pub fn branch(&mut self) -> Self {
        Self(self.0.branch())
    }

    pub fn push(&mut self, value: Value) {
        self.0.push(InternalValue::Value(value));
    }

    pub fn pop(&mut self) -> Result<Value, Error> {
        self.0
            .pop()
            .ok_or(Error::InternalRuntimeError)
            .and_then(InternalValue::try_into_value)
    }

    pub fn at(&self, index: usize) -> Result<Value, Error> {
        self.0
            .at(index)
            .ok_or(Error::InternalRuntimeError)
            .and_then(InternalValue::try_into_value)
    }

    pub fn pop_pointer(&mut self) -> Result<usize, Error> {
        self.0
            .pop()
            .ok_or(Error::InternalRuntimeError)
            .and_then(InternalValue::try_into_pointer)
    }

    pub fn replace_with_pointer(&mut self, index: usize, pointer: usize) -> Result<Value, Error> {
        self.0
            .replace_at(index, InternalValue::Pointer(pointer))
            .map_err(|_| Error::InternalRuntimeError)
            .and_then(InternalValue::try_into_value)
    }

    pub fn replace_with_value(&mut self, index: usize, value: Value) -> Result<usize, Error> {
        self.0
            .replace_at(index, InternalValue::Value(value))
            .map_err(|_| Error::InternalRuntimeError)
            .and_then(InternalValue::try_into_pointer)
    }

    pub fn replace_at(&mut self, index: usize, value: Value) -> Result<Value, Error> {
        self.0
            .replace_at(index, InternalValue::Value(value))
            .map_err(|_| Error::InternalRuntimeError)
            .and_then(InternalValue::try_into_value)
    }
}
