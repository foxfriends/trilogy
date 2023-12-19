use super::error::InternalRuntimeError;
use crate::cactus::{Branch, Cactus, Slice};
use crate::Value;
use std::fmt::{self, Display};

mod cont;
mod stack_cell;
mod stack_frame;
mod trace;

pub(crate) use cont::Cont;
pub use stack_cell::StackCell;
pub(crate) use stack_frame::StackFrame;
pub use trace::{StackTrace, StackTraceEntry};

/// The stack implementation for the Trilogy VM.
///
/// The Trilogy VM is backed by a cactus stack, the core of which is implemented as [`Cactus`][].
/// This wrapper around that base cactus implements the operations used in the execution of
/// Trilogy VM bytecode.
#[derive(Clone, Debug)]
pub(crate) struct Stack<'a> {
    frames: Vec<StackFrame<'a>>,
    branch: Branch<'a, StackCell>,
    fp: usize,
}

impl Display for Stack<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let items = self.branch.iter().collect::<Vec<_>>();
        if items.is_empty() {
            return write!(f, "<empty stack>");
        }
        for (i, item) in items.iter().enumerate() {
            if i != 0 {
                writeln!(f)?;
            }
            write!(f, "{}: {}", i, item)?;
        }
        Ok(())
    }
}

impl<'a> Stack<'a> {
    #[inline]
    pub(super) fn new(cactus: Branch<'a, StackCell>) -> Self {
        Self {
            frames: vec![],
            branch: cactus,
            fp: 0,
        }
    }

    #[inline]
    pub(crate) fn from_parts(
        frames: Vec<StackFrame<'a>>,
        cactus: Branch<'a, StackCell>,
        fp: usize,
    ) -> Self {
        Self {
            frames,
            branch: cactus,
            fp,
        }
    }

    #[inline]
    pub(crate) fn into_parts(self) -> (Vec<StackFrame<'a>>, Branch<'a, StackCell>, usize) {
        (self.frames, self.branch, self.fp)
    }

    #[inline]
    pub(super) fn cactus(&self) -> &'a Cactus<StackCell> {
        self.branch.cactus()
    }

    #[inline(always)]
    pub(super) fn branch(&mut self) -> Self {
        Self {
            frames: self.frames.clone(),
            branch: self.branch.branch(),
            fp: self.fp,
        }
    }

    #[inline(always)]
    pub(super) fn commit(&mut self) -> Slice<'a, StackCell> {
        self.branch.commit()
    }

    #[inline]
    pub(crate) fn frames(&self) -> impl Iterator<Item = &StackFrame<'a>> {
        self.frames.iter()
    }

    #[inline(always)]
    pub(super) fn push_unset(&mut self) {
        self.branch.push(StackCell::Unset);
    }

    #[inline(always)]
    pub(super) fn push<V>(&mut self, value: V)
    where
        V: Into<Value>,
    {
        self.branch.push(StackCell::Set(value.into()));
    }

    /// Pushes many values at once, not reversing their order as they would be if they
    /// were each pushed individually.
    #[inline(always)]
    pub(super) fn push_many(&mut self, mut values: Vec<StackCell>) {
        self.branch.append(&mut values);
    }

    #[inline(always)]
    pub(super) fn pop(&mut self) -> Result<Value, InternalRuntimeError> {
        self.branch
            .pop()
            .ok_or(InternalRuntimeError::ExpectedValue("empty stack"))?
            .into_set()
            .ok_or(InternalRuntimeError::ExpectedValue("empty cell"))
    }

    #[inline(always)]
    pub(super) fn peek(&mut self) -> Result<Value, InternalRuntimeError> {
        self.peek_raw()?
            .into_set()
            .ok_or(InternalRuntimeError::ExpectedValue("empty cell"))
    }

    #[inline(always)]
    pub(super) fn peek_raw(&mut self) -> Result<StackCell, InternalRuntimeError> {
        self.branch
            .peek()
            .ok_or(InternalRuntimeError::ExpectedValue("empty stack"))
    }

    #[inline(always)]
    pub(super) fn prepare_to_pop(&mut self, _count: usize) {
        // self.cactus.consume_exact(count);
    }

    #[inline(always)]
    fn reserve(&mut self, additional: usize) {
        self.branch.reserve(additional);
    }

    #[inline(always)]
    pub(super) fn slide(&mut self, count: usize) -> Result<(), InternalRuntimeError> {
        self.branch.consume_to_length(count + 1);
        let top = self.pop()?;
        let slide = self.pop_n(count)?;
        self.reserve(count);
        self.push(top);
        self.push_many(slide);
        Ok(())
    }

    #[inline(always)]
    pub(super) fn get(&self, index: usize) -> Result<Value, InternalRuntimeError> {
        self.get_raw(self.fp + index)?
            .into_set()
            .ok_or(InternalRuntimeError::ExpectedValue("empty cell"))
    }

    #[inline(always)]
    pub(super) fn get_raw(&self, index: usize) -> Result<StackCell, InternalRuntimeError> {
        self.branch
            .get(self.fp + index)
            .ok_or(InternalRuntimeError::ExpectedValue("out of bounds"))
    }

    #[inline(always)]
    pub(super) fn is_set(&self, index: usize) -> Result<bool, InternalRuntimeError> {
        Ok(self.get_raw(self.fp + index)?.is_set())
    }

    #[inline(always)]
    pub(super) fn pop_frame(&mut self) -> Result<Cont, InternalRuntimeError> {
        let frame = self
            .frames
            .pop()
            .ok_or(InternalRuntimeError::ExpectedReturn)?;
        if let Some(cactus) = frame.cactus {
            self.branch = Branch::from(cactus);
        }
        Ok(frame.cont)
    }

    #[inline(always)]
    pub(super) fn push_frame<C: Into<Cont>>(
        &mut self,
        c: C,
        arguments: Vec<StackCell>,
        stack: Option<Slice<'a, StackCell>>,
    ) {
        let fp = self.fp;
        let return_stack = match stack {
            None => {
                self.fp = self.branch.len();
                None
            }
            Some(stack) => {
                self.fp = 0;
                let mut branch = std::mem::replace(&mut self.branch, Branch::from(stack));
                Some(branch.commit())
            }
        };
        self.frames.push(StackFrame {
            cactus: return_stack,
            cont: c.into(),
            fp,
        });
        self.push_many(arguments);
    }

    #[inline(always)]
    pub(super) fn set(&mut self, index: usize, value: Value) {
        self.branch.set(self.fp + index, StackCell::Set(value));
    }

    #[inline(always)]
    pub(super) fn unset(&mut self, index: usize) {
        self.branch.set(self.fp + index, StackCell::Unset);
    }

    #[inline(always)]
    pub(super) fn init(
        &mut self,
        index: usize,
        value: Value,
    ) -> Result<bool, InternalRuntimeError> {
        if self.is_set(index)? {
            return Ok(false);
        }
        self.set(index, value);
        Ok(true)
    }

    /// Pops `n` values from the stack at once, returning them in an array __not__ in reverse order
    /// the way they would be if they were popped individually one after the other.
    #[inline(always)]
    pub(super) fn pop_n(&mut self, arity: usize) -> Result<Vec<StackCell>, InternalRuntimeError> {
        Ok(self.branch.pop_n(arity))
    }
}
