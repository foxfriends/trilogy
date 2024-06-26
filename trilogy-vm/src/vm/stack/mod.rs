use super::error::InternalRuntimeError;
use crate::cactus::{Branch, Slice, StackOverflow};
use crate::callable::{Closure, Continuation};
use crate::vm::stack::stack_dump::DumpCell;
use crate::{Offset, Value};
use std::collections::HashSet;
use std::fmt::{self, Display};

mod cont;
mod stack_cell;
mod stack_dump;
mod stack_frame;
mod trace;

pub(crate) use cont::Cont;
pub use stack_cell::StackCell;
pub use stack_dump::StackDump;
pub(crate) use stack_frame::StackFrame;
pub use trace::StackTrace;

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
    pub(crate) fn dump(&self) -> StackDump {
        let mut frames = self
            .frames
            .iter()
            .rev()
            .take_while(|frame| frame.slice.is_none())
            .map(|frame| frame.fp)
            .collect::<HashSet<_>>();
        frames.insert(self.fp);
        self.branch
            .iter()
            .map(Into::into)
            .enumerate()
            .flat_map(|(i, cell)| {
                if frames.contains(&i) {
                    vec![DumpCell::Frame, cell]
                } else {
                    vec![cell]
                }
            })
            .collect()
    }

    #[inline]
    pub(super) fn closure(&mut self, ip: Offset) -> Result<Closure, StackOverflow> {
        self.commit()?;
        let slice = self.branch.slice().slice(self.fp..self.branch.len());
        Ok(Closure::new(ip, slice))
    }

    #[inline]
    pub(super) fn continuation(&mut self, ip: Offset) -> Result<Continuation, StackOverflow> {
        let stack = self.branch()?;
        Continuation::new(ip, stack)
    }

    #[inline]
    pub(super) fn branch(&mut self) -> Result<Self, StackOverflow> {
        Ok(Self {
            frames: self.frames.clone(),
            branch: self.branch.branch()?,
            fp: self.fp,
        })
    }

    #[inline]
    pub(crate) fn active(&self) -> &Branch<StackCell> {
        &self.branch
    }

    #[inline]
    pub(super) fn commit(&mut self) -> Result<(), StackOverflow> {
        self.branch.commit()
    }

    #[inline]
    pub(crate) fn frames(&self) -> impl DoubleEndedIterator<Item = &StackFrame<'a>> {
        self.frames.iter()
    }

    #[inline]
    pub(super) fn push_unset(&mut self) {
        self.branch.push(StackCell::Unset);
    }

    #[inline]
    pub(super) fn push<V>(&mut self, value: V)
    where
        V: Into<Value>,
    {
        self.branch.push(StackCell::Set(value.into()));
    }

    /// Pushes many values at once, not reversing their order as they would be if they
    /// were each pushed individually.
    #[inline]
    pub(super) fn push_many(&mut self, mut values: Vec<StackCell>) {
        self.branch.append(&mut values);
    }

    #[inline]
    pub(super) fn pop(&mut self) -> Result<Value, InternalRuntimeError> {
        self.branch
            .pop()
            .ok_or(InternalRuntimeError::ExpectedValue("empty stack"))?
            .into_set()
            .ok_or(InternalRuntimeError::ExpectedValue("empty cell"))
    }

    #[inline]
    pub(super) fn pop_discard(&mut self) -> Result<(), InternalRuntimeError> {
        self.branch
            .pop()
            .ok_or(InternalRuntimeError::ExpectedValue("empty stack"))?;
        Ok(())
    }

    #[inline]
    pub(super) fn peek(&mut self) -> Result<Value, InternalRuntimeError> {
        self.peek_raw()?
            .into_set()
            .ok_or(InternalRuntimeError::ExpectedValue("empty cell"))
    }

    #[inline]
    pub(super) fn peek_raw(&mut self) -> Result<StackCell, InternalRuntimeError> {
        self.branch
            .peek()
            .ok_or(InternalRuntimeError::ExpectedValue("empty stack"))
    }

    #[inline]
    pub(super) fn prepare_to_pop(&mut self, count: usize) {
        self.branch.consume_to_length(count);
    }

    #[inline]
    fn reserve(&mut self, additional: usize) {
        self.branch.reserve(additional);
    }

    #[inline]
    pub(super) fn slide(&mut self, count: usize) -> Result<(), InternalRuntimeError> {
        self.branch.consume_to_length(count + 1);
        let top = self.pop()?;
        let slide = self.pop_n(count)?;
        self.reserve(count);
        self.push(top);
        self.push_many(slide);
        Ok(())
    }

    #[inline]
    pub(super) fn get(&self, index: usize) -> Result<Value, InternalRuntimeError> {
        self.get_raw(index)?
            .into_set()
            .ok_or(InternalRuntimeError::ExpectedValue("empty cell"))
    }

    #[inline]
    pub(super) fn get_raw(&self, index: usize) -> Result<StackCell, InternalRuntimeError> {
        self.branch
            .get(self.fp + index)
            .ok_or(InternalRuntimeError::ExpectedValue("out of bounds"))
    }

    #[inline]
    pub(super) fn is_set(&self, index: usize) -> Result<bool, InternalRuntimeError> {
        Ok(self.get_raw(index)?.is_set())
    }

    #[inline]
    pub(super) fn pop_frame(&mut self) -> Result<Cont, InternalRuntimeError> {
        let frame = self
            .frames
            .pop()
            .ok_or(InternalRuntimeError::ExpectedReturn)?;
        if let Some(slice) = frame.slice {
            let new_branch = Branch::from(slice);
            self.branch = new_branch;
        } else {
            self.branch.truncate(self.fp);
        }
        self.fp = frame.fp;
        Ok(frame.cont)
    }

    #[inline]
    pub(super) fn push_frame<C: Into<Cont>>(
        &mut self,
        c: C,
        arguments: Vec<StackCell>,
        stack: Option<Slice<'a, StackCell>>,
        here: Option<Offset>,
    ) -> Result<(), StackOverflow> {
        let fp = self.fp;
        let return_stack = match stack {
            None => {
                self.fp = self.branch.len();
                None
            }
            Some(stack) => {
                self.fp = 0;
                let mut branch = std::mem::replace(&mut self.branch, Branch::from(stack));
                branch.commit()?;
                Some(branch.into_slice())
            }
        };
        self.frames.push(StackFrame {
            slice: return_stack,
            cont: c.into(),
            fp,
            here,
        });
        self.push_many(arguments);
        Ok(())
    }

    #[inline]
    pub(super) fn set(&mut self, index: usize, value: Value) {
        self.branch.set(self.fp + index, StackCell::Set(value));
    }

    #[inline]
    pub(super) fn unset(&mut self, index: usize) {
        self.branch.set(self.fp + index, StackCell::Unset);
    }

    #[inline]
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
    #[inline]
    pub(super) fn pop_n(&mut self, arity: usize) -> Result<Vec<StackCell>, InternalRuntimeError> {
        self.branch
            .pop_n(arity)
            .ok_or(InternalRuntimeError::ExpectedValue("less than requested"))
    }
}
