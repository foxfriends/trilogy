use super::error::InternalRuntimeError;
use super::execution::Cont;
use crate::Value;
use crate::{cactus::Cactus, Offset};
use std::fmt::{self, Debug, Display};

mod internal_value;
mod trace;

use internal_value::Return;

pub(super) use internal_value::InternalValue;
pub use trace::{StackTrace, StackTraceEntry};

/// The stack implementation for the Trilogy VM.
///
/// The Trilogy VM is backed by a cactus stack, the core of which is implemented as [`Cactus`][].
/// This wrapper around that base cactus implements the operations used in the execution of
/// Trilogy VM bytecode.
#[derive(Clone, Default)]
pub(crate) struct Stack {
    /// The actual cactus that backs this stack.
    cactus: Cactus<InternalValue>,
    /// The size of the ghost stack's frame. The ghost stack is the closed-over stack of
    /// a closure, which is visible from closure calls. The closure returns onto the live
    /// stack, but has access to variables on the ghost.
    ghost_frame: usize,
    /// The size of the current stack frame. This is the offset at which the return pointer
    /// is written to the stack, to which the stack falls back to when a frame is popped.
    frame: usize,
}

impl Display for Stack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let items = self.cactus.iter().collect::<Vec<_>>();
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
            .iter()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .fold(&mut tuple, |f, v| f.field(&v))
            .finish()
    }
}

impl Stack {
    pub(super) fn new() -> Self {
        Self {
            cactus: Cactus::with_capacity(16),
            ghost_frame: 0,
            frame: 0,
        }
    }

    #[inline(always)]
    pub(super) fn branch(&mut self) -> Self {
        Self {
            cactus: self.cactus.branch(),
            ghost_frame: self.ghost_frame,
            frame: self.frame,
        }
    }

    #[inline(always)]
    pub(super) fn push_unset(&mut self) {
        self.cactus.push(InternalValue::Unset);
    }

    #[inline(always)]
    pub(super) fn push<V>(&mut self, value: V)
    where
        V: Into<Value>,
    {
        self.cactus.push(InternalValue::Value(value.into()));
    }

    /// Pushes many values at once, not reversing their order as they would be if they
    /// were each pushed individually.
    #[inline(always)]
    pub(super) fn push_many(&mut self, values: Vec<InternalValue>) {
        self.cactus.attach(values);
    }

    #[inline(always)]
    pub(super) fn pop(&mut self) -> Result<Option<Value>, InternalRuntimeError> {
        self.cactus
            .pop()
            .ok_or(InternalRuntimeError::ExpectedValue("empty stack"))
            .and_then(InternalValue::try_into_value_maybe)
    }

    #[inline(always)]
    pub(super) fn prepare_to_pop(&mut self, count: usize) {
        self.cactus.consume_exact(count);
    }

    #[inline(always)]
    fn reserve(&mut self, additional: usize) {
        self.cactus.reserve(additional);
    }

    #[inline(always)]
    pub(super) fn slide(&mut self, count: usize) -> Result<(), InternalRuntimeError> {
        self.cactus.consume_exact(count + 1);
        let top = self
            .pop()?
            .ok_or(InternalRuntimeError::ExpectedValue("empty stack"))?;
        let slide = self.pop_n(count)?;
        self.reserve(count);
        self.push(top);
        self.push_many(slide);
        Ok(())
    }

    #[inline(always)]
    pub(super) fn at(&self, index: usize) -> Result<Value, InternalRuntimeError> {
        self.cactus
            .at(index)
            .ok_or(InternalRuntimeError::ExpectedValue("out of bounds"))
            .and_then(InternalValue::try_into_value)
    }

    #[inline(always)]
    pub(super) fn at_raw(&self, index: usize) -> Result<InternalValue, InternalRuntimeError> {
        self.cactus
            .at(index)
            .ok_or(InternalRuntimeError::ExpectedValue("out of bounds"))
    }

    #[inline(always)]
    fn get_local_offset(&self, index: usize) -> Result<usize, InternalRuntimeError> {
        let locals = self.count_locals();
        if index >= locals {
            return Err(InternalRuntimeError::OutOfStackRange(index as Offset));
        }
        Ok(locals - index - 1)
    }

    #[inline(always)]
    pub(super) fn at_local(&self, index: usize) -> Result<Value, InternalRuntimeError> {
        let offset = self.get_local_offset(index)?;
        let local_locals = self.len() - self.frame;
        if offset >= local_locals {
            let val = self.cactus.at(self.len() - self.frame).unwrap();
            let ret = val.as_return().unwrap();
            let ghost = ret.ghost.as_ref().unwrap();
            return ghost.at_local(index);
        }
        self.cactus
            .at(offset)
            .ok_or(InternalRuntimeError::ExpectedValue("local out of bounds"))
            .and_then(InternalValue::try_into_value)
    }

    #[inline(always)]
    pub(super) fn is_set_local(&self, index: usize) -> Result<bool, InternalRuntimeError> {
        let offset = self.get_local_offset(index)?;
        let local_locals = self.len() - self.frame;
        if offset >= local_locals {
            let val = self.cactus.at(self.len() - self.frame).unwrap();
            let ret = val.as_return().unwrap();
            let ghost = ret.ghost.as_ref().unwrap();
            return ghost.is_set_local(index);
        }
        self.cactus
            .at(offset)
            .ok_or(InternalRuntimeError::ExpectedValue("local out of bounds"))
            .and_then(|val| val.is_set())
    }

    #[inline(always)]
    pub(super) fn pop_frame(&mut self) -> Result<Cont, InternalRuntimeError> {
        // TODO: thought using `discard` here would be faster than `detach_at`, but
        // practically it appears not. Maybe something is weird with the `discard`
        // implementation?
        self.cactus.consume_exact(self.len() - self.frame + 1);
        self.cactus.detach_at(self.len() - self.frame);
        let ret = self
            .cactus
            .pop()
            .unwrap()
            .into_return()
            .ok_or(InternalRuntimeError::ExpectedReturn)?;
        self.frame = ret.frame;
        self.ghost_frame = ret.ghost_frame;
        Ok(ret.cont)
    }

    #[inline(always)]
    pub(super) fn push_frame<C: Into<Cont>>(
        &mut self,
        c: C,
        arguments: Vec<InternalValue>,
        stack: Option<Stack>,
    ) {
        let frame = self.frame;
        let ghost_frame = self.ghost_frame;
        self.ghost_frame = stack.as_ref().map(|st| st.count_locals()).unwrap_or(0);
        self.cactus.push(InternalValue::ret(Return {
            cont: c.into(),
            frame,
            ghost_frame,
            ghost: stack,
        }));
        self.frame = self.len();
        self.push_many(arguments);
    }

    #[inline(always)]
    pub(super) fn set_local(
        &mut self,
        index: usize,
        value: Value,
    ) -> Result<Option<Value>, InternalRuntimeError> {
        let offset = self.get_local_offset(index)?;
        let local_locals = self.len() - self.frame;
        if offset >= local_locals {
            let val = self.cactus.at(self.len() - self.frame).unwrap();
            let ret = val.as_return().unwrap();
            let ghost = ret.ghost.as_ref().unwrap();
            return ghost.set_local_shared(index, value);
        }
        self.cactus
            .replace_at(offset, InternalValue::Value(value))
            .map_err(|_| InternalRuntimeError::ExpectedValue("local out of bounds"))
            .and_then(|val| val.try_into_value_maybe())
    }

    fn set_local_shared(
        &self,
        index: usize,
        value: Value,
    ) -> Result<Option<Value>, InternalRuntimeError> {
        let offset = self.get_local_offset(index)?;
        let local_locals = self.len() - self.frame;
        if offset >= local_locals {
            let val = self.cactus.at(self.len() - self.frame).unwrap();
            let ret = val.as_return().unwrap();
            let ghost = ret.ghost.as_ref().unwrap();
            return ghost.set_local_shared(index, value);
        }
        self.cactus
            .replace_shared(offset, InternalValue::Value(value))
            .map_err(|_| InternalRuntimeError::ExpectedValue("local out of bounds"))
            .and_then(|val| val.try_into_value_maybe())
    }

    #[inline(always)]
    pub(super) fn unset_local(
        &mut self,
        index: usize,
    ) -> Result<Option<Value>, InternalRuntimeError> {
        let offset = self.get_local_offset(index)?;
        let local_locals = self.len() - self.frame;
        if offset >= local_locals {
            let val = self.cactus.at(self.len() - self.frame).unwrap();
            let ret = val.as_return().unwrap();
            let ghost = ret.ghost.as_ref().unwrap();
            return ghost.unset_local_shared(index);
        }
        self.cactus
            .replace_at(offset, InternalValue::Unset)
            .map_err(|_| InternalRuntimeError::ExpectedValue("local out of bounds"))
            .and_then(|val| val.try_into_value_maybe())
    }

    fn unset_local_shared(&self, index: usize) -> Result<Option<Value>, InternalRuntimeError> {
        let offset = self.get_local_offset(index)?;
        let local_locals = self.len() - self.frame;
        if offset >= local_locals {
            let val = self.cactus.at(self.len() - self.frame).unwrap();
            let ret = val.as_return().unwrap();
            let ghost = ret.ghost.as_ref().unwrap();
            return ghost.unset_local_shared(index);
        }
        self.cactus
            .replace_shared(offset, InternalValue::Unset)
            .map_err(|_| InternalRuntimeError::ExpectedValue("local out of bounds"))
            .and_then(|val| val.try_into_value_maybe())
    }

    #[inline(always)]
    pub(super) fn init_local(
        &mut self,
        index: usize,
        value: Value,
    ) -> Result<bool, InternalRuntimeError> {
        if self.is_set_local(index)? {
            return Ok(false);
        }
        let offset = self.get_local_offset(index)?;
        let local_locals = self.len() - self.frame;
        if offset >= local_locals {
            let val = self.cactus.at(self.len() - self.frame).unwrap();
            let ret = val.as_return().unwrap();
            let ghost = ret.ghost.as_ref().unwrap();
            ghost.set_local_shared(index, value).unwrap();
            Ok(true)
        } else {
            self.cactus
                .replace_at(offset, InternalValue::Value(value))
                .unwrap();
            Ok(true)
        }
    }

    /// Pops `n` values from the stack at once, returning them in an array __not__ in reverse order
    /// the way they would be if they were popped individually one after the other.
    #[inline(always)]
    pub(super) fn pop_n(
        &mut self,
        arity: usize,
    ) -> Result<Vec<InternalValue>, InternalRuntimeError> {
        let internal_values = self.cactus.detach_at(arity);
        Ok(internal_values)
    }

    /// The full length of the live stack, including entries inaccessible to the VM
    /// at this time (e.g. cells in call frames beyond the current one).
    #[inline(always)]
    fn len(&self) -> usize {
        self.cactus.count()
    }

    /// The number of local offsets on the stack currently accessible by the VM. This
    /// includes the current stack frame and the frames of the ghost stack.
    ///
    /// A ghost stack may itself have parent ghost stacks and so on, all of which are
    /// reflected by the `ghost_frame`.
    #[inline(always)]
    fn count_locals(&self) -> usize {
        self.len() - self.frame + self.ghost_frame
    }
}