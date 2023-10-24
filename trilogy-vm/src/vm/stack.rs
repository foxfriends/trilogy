use super::error::InternalRuntimeError;
use super::execution::Cont;
use crate::cactus::Cactus;
use crate::Value;
use std::fmt::{self, Debug, Display};

#[derive(Clone, Debug)]
pub(super) enum InternalValue {
    Unset,
    Value(Value),
    Return {
        cont: Cont,
        frame: usize,
        ghost: Option<Stack>,
    },
}

impl InternalValue {
    pub fn try_into_value(self) -> Result<Value, InternalRuntimeError> {
        match self {
            InternalValue::Value(value) => Ok(value),
            InternalValue::Unset => Err(InternalRuntimeError::ExpectedValue("empty cell")),
            InternalValue::Return { .. } => {
                Err(InternalRuntimeError::ExpectedValue("return pointer"))
            }
        }
    }

    fn try_into_value_maybe(self) -> Result<Option<Value>, InternalRuntimeError> {
        match self {
            InternalValue::Value(value) => Ok(Some(value)),
            InternalValue::Unset => Ok(None),
            InternalValue::Return { .. } => {
                Err(InternalRuntimeError::ExpectedValue("return pointer"))
            }
        }
    }

    fn is_set(&self) -> Result<bool, InternalRuntimeError> {
        match self {
            InternalValue::Value(..) => Ok(true),
            InternalValue::Unset => Ok(false),
            InternalValue::Return { .. } => {
                Err(InternalRuntimeError::ExpectedValue("return pointer"))
            }
        }
    }
}

impl Display for InternalValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InternalValue::Unset => write!(f, "<unset>"),
            InternalValue::Value(value) => write!(f, "{value}"),
            InternalValue::Return {
                cont, ghost: None, ..
            } => write!(f, "-> {cont:?}"),
            InternalValue::Return {
                cont,
                ghost: Some(ghost),
                ..
            } => {
                let ghost_str = format!("{}", ghost)
                    .lines()
                    .map(|line| format!("\t{line}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                writeln!(f, "{}", ghost_str)?;
                write!(f, "-> {cont:?}\t[closure]")
            }
        }
    }
}

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

#[derive(Clone, Debug)]
pub struct Caller {
    pub ip: usize,
    pub name: Option<String>,
}

#[derive(Clone, Debug)]
pub struct StackFrame {
    pub caller: Caller,
    pub callee: Caller,
    pub exit_at: usize,
}

#[derive(Clone, Debug)]
pub struct StackTrace {
    pub frames: Vec<StackFrame>,
    pub ip: usize,
}

impl Display for StackTrace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (
            i,
            StackFrame {
                caller,
                callee,
                exit_at,
            },
        ) in self.frames.iter().rev().enumerate()
        {
            writeln!(
                f,
                "{i}. {}:{}",
                callee.ip,
                callee.name.as_deref().unwrap_or("<unknown>")
            )?;
            writeln!(
                f,
                "\tat {}:{}[{}]",
                caller.ip,
                caller.name.as_deref().unwrap_or("<unknown>"),
                exit_at,
            )?;
        }
        writeln!(f, "Final IP: {}", self.ip)
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

    pub(super) fn at_local(&self, index: usize) -> Result<Value, InternalRuntimeError> {
        let register = self.count_locals() - index - 1;
        let local_locals = self.len() - self.frame;
        if register >= local_locals {
            let InternalValue::Return {
                ghost: Some(stack), ..
            } = self.cactus.at(self.len() - self.frame).unwrap()
            else {
                panic!()
            };
            return stack.at_local(index);
        }
        self.cactus
            .at(register)
            .ok_or(InternalRuntimeError::ExpectedValue("local out of bounds"))
            .and_then(InternalValue::try_into_value)
    }

    pub(super) fn is_set_local(&self, index: usize) -> Result<bool, InternalRuntimeError> {
        let register = self.count_locals() - index - 1;
        let local_locals = self.len() - self.frame;
        if register >= local_locals {
            let InternalValue::Return {
                ghost: Some(stack), ..
            } = self.cactus.at(self.len() - self.frame).unwrap()
            else {
                panic!()
            };
            return stack.is_set_local(index);
        }
        self.cactus
            .at(register)
            .ok_or(InternalRuntimeError::ExpectedValue("local out of bounds"))
            .and_then(|val| val.is_set())
    }

    pub(super) fn pop_frame(&mut self) -> Result<Cont, InternalRuntimeError> {
        loop {
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
            ghost: stack,
        });
        self.frame = self.len();
        self.push_many(arguments);
    }

    pub(super) fn set_local(
        &mut self,
        index: usize,
        value: Value,
    ) -> Result<Option<Value>, InternalRuntimeError> {
        let register = self.count_locals() - index - 1;
        let local_locals = self.len() - self.frame;
        if register >= local_locals {
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
            return stack.set_local(index, value);
        }
        self.cactus
            .replace_at(register, InternalValue::Value(value))
            .map_err(|_| InternalRuntimeError::ExpectedValue("local out of bounds"))
            .and_then(|val| val.try_into_value_maybe())
    }

    pub(super) fn unset_local(
        &mut self,
        index: usize,
    ) -> Result<Option<Value>, InternalRuntimeError> {
        let register = self.count_locals() - index - 1;
        let local_locals = self.len() - self.frame;
        if register >= local_locals {
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
            return stack.unset_local(index);
        }
        self.cactus
            .replace_at(register, InternalValue::Unset)
            .map_err(|_| InternalRuntimeError::ExpectedValue("local out of bounds"))
            .and_then(|val| val.try_into_value_maybe())
    }

    pub(super) fn init_local(
        &mut self,
        index: usize,
        value: Value,
    ) -> Result<bool, InternalRuntimeError> {
        let register = self.count_locals() - index - 1;
        let local_locals = self.len() - self.frame;
        if register >= local_locals {
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
            return stack.init_local(index, value);
        }
        if matches!(self.cactus.at(register), Some(InternalValue::Unset)) {
            self.cactus
                .replace_at(register, InternalValue::Value(value))
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
                ghost: Some(stack), ..
            }) => stack.count_locals() + local_locals,
            _ => local_locals,
        }
    }
}
