use super::error::InternalRuntimeError;
use crate::{cactus::Cactus, Program, Value};
use std::collections::BTreeMap;
use std::fmt::{self, Debug, Display};

#[derive(Clone, Debug)]
enum InternalValue {
    Value(Value),
    Return { ip: usize, frame: usize },
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

impl Display for InternalValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InternalValue::Value(value) => write!(f, "{value}"),
            InternalValue::Return { ip, .. } => write!(f, "-> {ip}"),
            InternalValue::Pointer(offset) => write!(f, "&{offset}"),
            InternalValue::Stack(stack) => write!(f, "reset ({})", stack.cactus.at(0).unwrap()),
        }
    }
}

#[derive(Default, Clone)]
pub struct Stack {
    cactus: Cactus<InternalValue>,
    frame: usize,
}

impl Display for Stack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, item) in self
            .cactus
            .clone()
            .into_iter()
            .collect::<Vec<_>>()
            .into_iter()
            .enumerate()
            .rev()
        {
            writeln!(f, "{}: {}", i, item)?;
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
    pub fn trace(&self, from_ip: usize, program: &Program) -> StackTrace {
        let mut ip_history: Vec<usize> = vec![from_ip];
        self.trace_into(&mut ip_history);

        let mut directory = BTreeMap::<usize, Vec<&str>>::new();
        for (label, ip) in &program.labels {
            directory.entry(*ip).or_default().push(label);
        }

        let frames = ip_history
            .windows(2)
            .map(|window| {
                let exit_at = window[0];
                let jump_from = window[1];
                let callee = directory
                    .range(..exit_at)
                    .last()
                    .map(|(&ip, labels)| Caller {
                        ip,
                        name: labels.first().map(|&s| s.to_owned()),
                    })
                    .unwrap_or(Caller { ip: 0, name: None });
                let caller = directory
                    .range(..jump_from)
                    .last()
                    .map(|(&ip, labels)| Caller {
                        ip,
                        name: labels.first().map(|&s| s.to_owned()),
                    })
                    .unwrap_or(Caller { ip: 0, name: None });
                StackFrame {
                    caller,
                    callee,
                    exit_at,
                }
            })
            .collect();

        StackTrace {
            frames,
            ip: from_ip,
        }
    }

    fn trace_into(&self, ip_history: &mut Vec<usize>) {
        for value in self.cactus.clone() {
            match value {
                InternalValue::Return { ip, .. } => {
                    ip_history.push(ip);
                }
                InternalValue::Stack(stack) => stack.trace_into(ip_history),
                _ => {}
            }
        }
    }

    pub(crate) fn branch(&mut self) -> Self {
        Self {
            cactus: self.cactus.branch(),
            frame: self.frame,
        }
    }

    pub(crate) fn push(&mut self, value: Value) {
        self.cactus.push(InternalValue::Value(value));
    }

    pub(crate) fn pop(&mut self) -> Result<Value, InternalRuntimeError> {
        self.cactus
            .pop()
            .ok_or(InternalRuntimeError::ExpectedValue)
            .and_then(InternalValue::try_into_value)
    }

    pub(crate) fn at(&self, index: usize) -> Result<Value, InternalRuntimeError> {
        self.cactus
            .at(index)
            .ok_or(InternalRuntimeError::ExpectedValue)
            .and_then(InternalValue::try_into_value)
    }

    pub(crate) fn at_local(&self, index: usize) -> Result<Value, InternalRuntimeError> {
        self.cactus
            .at(self.len() - 1 - self.frame - index)
            .ok_or(InternalRuntimeError::ExpectedValue)
            .and_then(InternalValue::try_into_value)
    }

    pub(crate) fn pop_pointer(&mut self) -> Result<usize, InternalRuntimeError> {
        self.cactus
            .pop()
            .ok_or(InternalRuntimeError::ExpectedPointer)
            .and_then(InternalValue::try_into_pointer)
    }

    pub(crate) fn pop_return(&mut self) -> Result<usize, InternalRuntimeError> {
        loop {
            let popped = self
                .cactus
                .pop()
                .ok_or(InternalRuntimeError::ExpectedReturn)?;
            if let InternalValue::Return { ip, frame } = popped {
                self.frame = frame;
                return Ok(ip);
            }
        }
    }

    pub(crate) fn push_pointer(&mut self, pointer: usize) {
        self.cactus.push(InternalValue::Pointer(pointer));
    }

    pub(crate) fn replace_with_return(
        &mut self,
        index: usize,
        ip: usize,
    ) -> Result<Value, InternalRuntimeError> {
        let callable = self
            .cactus
            .replace_at(
                index,
                InternalValue::Return {
                    ip,
                    frame: self.frame,
                },
            )
            .map_err(|_| InternalRuntimeError::ExpectedValue)
            .and_then(InternalValue::try_into_value)?;
        if matches!(callable, Value::Procedure(..)) {
            self.frame = self.len() - index;
        }
        Ok(callable)
    }

    pub(crate) fn replace_at(
        &mut self,
        index: usize,
        value: Value,
    ) -> Result<Value, InternalRuntimeError> {
        self.cactus
            .replace_at(index, InternalValue::Value(value))
            .map_err(|_| InternalRuntimeError::ExpectedValue)
            .and_then(InternalValue::try_into_value)
    }

    pub(crate) fn replace_at_local(
        &mut self,
        index: usize,
        value: Value,
    ) -> Result<Value, InternalRuntimeError> {
        self.cactus
            .replace_at(
                self.len() - 1 - self.frame - index,
                InternalValue::Value(value),
            )
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
            .cactus
            .detach_at(offset)
            .ok_or(InternalRuntimeError::ExpectedValue)?;
        let return_to = std::mem::replace(self, stack);
        self.cactus.push(InternalValue::Stack(return_to));
        self.cactus.attach(transfer);
        Ok(())
    }

    pub(crate) fn return_to(&mut self) -> Result<(), InternalRuntimeError> {
        while let Some(value) = self.cactus.pop() {
            if let InternalValue::Stack(stack) = value {
                *self = stack;
                return Ok(());
            }
        }
        Err(InternalRuntimeError::ExpectedStack)
    }

    pub(crate) fn len(&self) -> usize {
        self.cactus.len()
    }
}
