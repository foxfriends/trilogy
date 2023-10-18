use super::error::InternalRuntimeError;
use crate::bytecode::Offset;
use crate::{cactus::Cactus, Value};
use std::fmt::{self, Debug, Display};

#[derive(Clone, Debug)]
pub(crate) enum InternalValue {
    Unset,
    Value(Value),
    Return {
        ip: Offset,
        frame: usize,
        ghost: Option<Stack>,
    },
    Pointer(usize),
}

impl InternalValue {
    pub(crate) fn try_into_value(self) -> Result<Value, InternalRuntimeError> {
        match self {
            InternalValue::Value(value) => Ok(value),
            InternalValue::Unset => Err(InternalRuntimeError::ExpectedValue("empty cell")),
            InternalValue::Return { .. } => {
                Err(InternalRuntimeError::ExpectedValue("return pointer"))
            }
            InternalValue::Pointer(..) => Err(InternalRuntimeError::ExpectedValue("pointer")),
        }
    }

    fn try_into_value_maybe(self) -> Result<Option<Value>, InternalRuntimeError> {
        match self {
            InternalValue::Value(value) => Ok(Some(value)),
            InternalValue::Unset => Ok(None),
            InternalValue::Return { .. } => {
                Err(InternalRuntimeError::ExpectedValue("return pointer"))
            }
            InternalValue::Pointer(..) => Err(InternalRuntimeError::ExpectedValue("pointer")),
        }
    }

    fn is_set(&self) -> Result<bool, InternalRuntimeError> {
        match self {
            InternalValue::Value(..) => Ok(true),
            InternalValue::Unset => Ok(false),
            InternalValue::Return { .. } => {
                Err(InternalRuntimeError::ExpectedValue("return pointer"))
            }
            InternalValue::Pointer(..) => Err(InternalRuntimeError::ExpectedValue("pointer")),
        }
    }

    fn try_into_pointer(self) -> Result<usize, InternalRuntimeError> {
        match self {
            InternalValue::Pointer(pointer) => Ok(pointer),
            _ => Err(InternalRuntimeError::ExpectedPointer),
        }
    }
}

impl Display for InternalValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InternalValue::Unset => write!(f, "<unset>"),
            InternalValue::Value(value) => write!(f, "{value}"),
            InternalValue::Return {
                ip, ghost: None, ..
            } => write!(f, "-> {ip}"),
            InternalValue::Return {
                ip,
                ghost: Some(ghost),
                ..
            } => {
                let ghost_str = format!("{}", ghost)
                    .lines()
                    .map(|line| format!("\t{line}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                writeln!(f, "{}", ghost_str)?;
                write!(f, "-> {ip}\t[closure]")
            }
            InternalValue::Pointer(offset) => write!(f, "&{offset}"),
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
    pub(crate) fn branch(&mut self) -> Self {
        Self {
            cactus: self.cactus.branch(),
            frame: self.frame,
        }
    }

    pub(crate) fn push_unset(&mut self) {
        self.cactus.push(InternalValue::Unset);
    }

    pub(crate) fn push(&mut self, value: Value) {
        self.cactus.push(InternalValue::Value(value));
    }

    /// Pushes many values at once, not reversing their order as they would be if they
    /// were each pushed individually.
    pub(crate) fn push_many(&mut self, values: Vec<InternalValue>) {
        self.cactus.attach(values);
    }

    pub(crate) fn pop(&mut self) -> Result<Option<Value>, InternalRuntimeError> {
        self.cactus
            .pop()
            .ok_or(InternalRuntimeError::ExpectedValue("empty stack"))
            .and_then(InternalValue::try_into_value_maybe)
    }

    pub(crate) fn at(&self, index: usize) -> Result<Value, InternalRuntimeError> {
        self.cactus
            .at(index)
            .ok_or(InternalRuntimeError::ExpectedValue("out of bounds"))
            .and_then(InternalValue::try_into_value)
    }

    pub(crate) fn at_local(&self, index: usize) -> Result<Value, InternalRuntimeError> {
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

    pub(crate) fn is_set_local(&self, index: usize) -> Result<bool, InternalRuntimeError> {
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

    pub(crate) fn pop_pointer(&mut self) -> Result<usize, InternalRuntimeError> {
        self.cactus
            .pop()
            .ok_or(InternalRuntimeError::ExpectedPointer)
            .and_then(InternalValue::try_into_pointer)
    }

    pub(crate) fn pop_frame(&mut self) -> Result<Offset, InternalRuntimeError> {
        loop {
            let popped = self
                .cactus
                .pop()
                .ok_or(InternalRuntimeError::ExpectedReturn)?;
            if let InternalValue::Return { ip, frame, .. } = popped {
                self.frame = frame;
                return Ok(ip);
            }
        }
    }

    pub(crate) fn push_frame(
        &mut self,
        ip: Offset,
        arguments: Vec<InternalValue>,
        stack: Option<Stack>,
    ) {
        let frame = self.frame;
        self.cactus.push(InternalValue::Return {
            ip,
            frame,
            ghost: stack,
        });
        self.frame = self.len();
        self.push_many(arguments);
    }

    pub(crate) fn push_pointer(&mut self, pointer: usize) {
        self.cactus.push(InternalValue::Pointer(pointer));
    }

    pub(crate) fn set_local(
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

    pub(crate) fn unset_local(
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

    pub(crate) fn init_local(
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
    pub(crate) fn pop_n(
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
