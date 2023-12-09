use super::error::InternalRuntimeError;
use super::execution::Cont;
use crate::cactus::Cactus;
use crate::Value;
use std::fmt::{self, Debug, Display};

mod ghost;
mod internal_value;
mod trace;

use ghost::Ghost;
pub(super) use internal_value::InternalValue;
pub use trace::{StackTrace, StackTraceEntry};

#[derive(Default, Clone)]
pub(crate) struct Stack {
    cactus: Cactus<InternalValue>,
    frame: usize,
}

impl Display for Stack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let items = self.cactus.clone().into_iter().collect::<Vec<_>>();
        for (i, item) in items.iter().enumerate().rev() {
            if i != items.len() - 1 {
                writeln!(f)?;
            }
            write!(f, "{}: {}", i, item)?;
        }
        Ok(())
    }
}

impl Debug for Stack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tuple = f.debug_tuple("Stack");
        self.cactus
            .clone()
            .into_iter()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .fold(&mut tuple, |f, v| f.field(&v))
            .finish()
    }
}

impl Stack {
    pub(super) fn branch(&mut self) -> Self {
        Self {
            cactus: self.cactus.branch(),
            frame: self.frame,
        }
    }

    pub(super) fn push_unset(&mut self) {
        self.cactus.push(InternalValue::Unset);
    }

    pub(super) fn push<V>(&mut self, value: V)
    where
        V: Into<Value>,
    {
        self.cactus.push(InternalValue::Value(value.into()));
    }

    /// Pushes many values at once, not reversing their order as they would be if they
    /// were each pushed individually.
    pub(super) fn push_many(&mut self, values: Vec<InternalValue>) {
        self.cactus.attach(values);
    }

    pub(super) fn pop(&mut self) -> Result<Option<Value>, InternalRuntimeError> {
        self.cactus
            .pop()
            .ok_or(InternalRuntimeError::ExpectedValue("empty stack"))
            .and_then(InternalValue::try_into_value_maybe)
    }

    pub(super) fn slide(&mut self, count: usize) -> Result<(), InternalRuntimeError> {
        let top = self
            .pop()?
            .ok_or(InternalRuntimeError::ExpectedValue("empty stack"))?;
        let slide = self.pop_n(count)?;
        self.push(top);
        self.push_many(slide);
        Ok(())
    }

    pub(super) fn at(&self, index: usize) -> Result<Value, InternalRuntimeError> {
        self.cactus
            .at(index)
            .ok_or(InternalRuntimeError::ExpectedValue("out of bounds"))
            .and_then(InternalValue::try_into_value)
    }

    pub(super) fn at_raw(&self, index: usize) -> Result<InternalValue, InternalRuntimeError> {
        self.cactus
            .at(index)
            .ok_or(InternalRuntimeError::ExpectedValue("out of bounds"))
    }

    fn get_local_offset(&self, index: usize) -> Result<usize, InternalRuntimeError> {
        let locals = self.count_locals();
        if index >= locals {
            return Err(InternalRuntimeError::OutOfStackRange(index as u32));
        }
        Ok(locals - index - 1)
    }

    pub(super) fn at_local(&self, index: usize) -> Result<Value, InternalRuntimeError> {
        let offset = self.get_local_offset(index)?;
        let local_locals = self.len() - self.frame;
        if offset >= local_locals {
            let InternalValue::Return {
                ghost: Some(ghost), ..
            } = self.cactus.at(self.len() - self.frame).unwrap()
            else {
                panic!()
            };
            return ghost.stack.at_local(index);
        }
        self.cactus
            .at(offset)
            .ok_or(InternalRuntimeError::ExpectedValue("local out of bounds"))
            .and_then(InternalValue::try_into_value)
    }

    pub(super) fn is_set_local(&self, index: usize) -> Result<bool, InternalRuntimeError> {
        let offset = self.get_local_offset(index)?;
        let local_locals = self.len() - self.frame;
        if offset >= local_locals {
            let InternalValue::Return {
                ghost: Some(ghost), ..
            } = self.cactus.at(self.len() - self.frame).unwrap()
            else {
                panic!()
            };
            return ghost.stack.is_set_local(index);
        }
        self.cactus
            .at(offset)
            .ok_or(InternalRuntimeError::ExpectedValue("local out of bounds"))
            .and_then(|val| val.is_set())
    }

    pub(super) fn pop_frame(&mut self) -> Result<Cont, InternalRuntimeError> {
        loop {
            // TODO: This is pretty inefficient! Popping repeatedly is sometimes slow, rather
            // pop in one big chunk.
            let popped = self
                .cactus
                .pop()
                .ok_or(InternalRuntimeError::ExpectedReturn)?;
            if let InternalValue::Return { cont, frame, .. } = popped {
                self.frame = frame;
                return Ok(cont);
            }
        }
    }

    pub(super) fn push_frame<C: Into<Cont>>(
        &mut self,
        c: C,
        arguments: Vec<InternalValue>,
        stack: Option<Stack>,
    ) {
        let frame = self.frame;
        self.cactus.push(InternalValue::Return {
            cont: c.into(),
            frame,
            ghost: stack.map(Ghost::from),
        });
        self.frame = self.len();
        self.push_many(arguments);
    }

    pub(super) fn set_local(
        &mut self,
        index: usize,
        value: Value,
    ) -> Result<Option<Value>, InternalRuntimeError> {
        let offset = self.count_locals() - index - 1;
        let local_locals = self.len() - self.frame;
        if offset >= local_locals {
            // NOTE: The `mut` (and requirement for `mut`) is sort of fake.
            //
            // The stack here just happens to be a cactus with nothing in its immediate list,
            // and everything in its parent. Editing it therefore occurs on the shared parent
            // which is in a mutex and does not require mutable access.
            //
            // Overall it's just a convenient coincidence coming from the weird way the cactus
            // is built. I hope someday a more explicitly shared stack representation is devised...
            // Unless this is actually correct and I just don't understand what I'm doing but it's working.
            let InternalValue::Return {
                ghost: Some(mut stack),
                ..
            } = self.cactus.at(self.len() - self.frame).unwrap()
            else {
                panic!()
            };
            return stack.stack.set_local(index, value);
        }
        self.cactus
            .replace_at(offset, InternalValue::Value(value))
            .map_err(|_| InternalRuntimeError::ExpectedValue("local out of bounds"))
            .and_then(|val| val.try_into_value_maybe())
    }

    pub(super) fn unset_local(
        &mut self,
        index: usize,
    ) -> Result<Option<Value>, InternalRuntimeError> {
        let offset = self.get_local_offset(index)?;
        let local_locals = self.len() - self.frame;
        if offset >= local_locals {
            // NOTE: The `mut` (and requirement for `mut`) is sort of fake.
            //
            // The stack here just happens to be a cactus with nothing in its immediate list,
            // and everything in its parent. Editing it therefore occurs on the shared parent
            // which is in a mutex and does not require mutable access.
            //
            // Overall it's just a convenient coincidence coming from the weird way the cactus
            // is built. I hope someday a more explicitly shared stack representation is devised...
            // Unless this is actually correct and I just don't understand what I'm doing but it's working.
            let InternalValue::Return {
                ghost: Some(mut ghost),
                ..
            } = self.cactus.at(self.len() - self.frame).unwrap()
            else {
                panic!()
            };
            return ghost.stack.unset_local(index);
        }
        self.cactus
            .replace_at(offset, InternalValue::Unset)
            .map_err(|_| InternalRuntimeError::ExpectedValue("local out of bounds"))
            .and_then(|val| val.try_into_value_maybe())
    }

    pub(super) fn init_local(
        &mut self,
        index: usize,
        value: Value,
    ) -> Result<bool, InternalRuntimeError> {
        let offset = self.get_local_offset(index)?;
        let local_locals = self.len() - self.frame;
        if offset >= local_locals {
            // NOTE: The `mut` (and requirement for `mut`) is sort of fake.
            //
            // The stack here just happens to be a cactus with nothing in its immediate list,
            // and everything in its parent. Editing it therefore occurs on the shared parent
            // which is in a mutex and does not require mutable access.
            //
            // Overall it's just a convenient coincidence coming from the weird way the cactus
            // is built. I hope someday a more explicitly shared stack representation is devised...
            // Unless this is actually correct and I just don't understand what I'm doing but it's working.
            let InternalValue::Return {
                ghost: Some(mut ghost),
                ..
            } = self.cactus.at(self.len() - self.frame).unwrap()
            else {
                panic!()
            };
            return ghost.stack.init_local(index, value);
        }
        if matches!(self.cactus.at(offset), Some(InternalValue::Unset)) {
            self.cactus
                .replace_at(offset, InternalValue::Value(value))
                .unwrap();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Pops `n` values from the stack at once, returning them in an array __not__ in reverse order
    /// the way they would be if they were popped individually one after the other.
    pub(super) fn pop_n(
        &mut self,
        arity: usize,
    ) -> Result<Vec<InternalValue>, InternalRuntimeError> {
        let internal_values = self
            .cactus
            .detach_at(arity)
            .ok_or(InternalRuntimeError::ExpectedValue("stack too short"))?;
        Ok(internal_values)
    }

    fn len(&self) -> usize {
        self.cactus.len()
    }

    fn count_locals(&self) -> usize {
        let local_locals = self.len() - self.frame;
        match self.cactus.at(self.len() - self.frame) {
            Some(InternalValue::Return {
                ghost: Some(ghost), ..
            }) => ghost.len + local_locals,
            _ => local_locals,
        }
    }
}
