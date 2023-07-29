use super::error::InternalRuntimeError;
use crate::{cactus::Cactus, Program, Value};
use std::collections::BTreeMap;
use std::fmt::{self, Debug, Display};

#[derive(Clone, Debug)]
enum InternalValue {
    Value(Value),
    Return {
        ip: usize,
        frame: usize,
        ghost: Option<Stack>,
    },
    Pointer(usize),
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
            if let InternalValue::Return { ip, .. } = value {
                ip_history.push(ip);
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

    pub(crate) fn push_many(&mut self, values: Vec<Value>) {
        self.cactus
            .attach(values.into_iter().map(InternalValue::Value).collect());
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
        let register = self.count_locals() - index - 1;
        let local_locals = self.len() - self.frame;
        if register >= local_locals {
            let InternalValue::Return{ ghost: Some(stack), .. } = self.cactus.at(self.len() - self.frame).unwrap() else { panic!() };
            return stack.at_local(index);
        }
        self.cactus
            .at(register)
            .ok_or(InternalRuntimeError::ExpectedValue)
            .and_then(InternalValue::try_into_value)
    }

    pub(crate) fn pop_pointer(&mut self) -> Result<usize, InternalRuntimeError> {
        self.cactus
            .pop()
            .ok_or(InternalRuntimeError::ExpectedPointer)
            .and_then(InternalValue::try_into_pointer)
    }

    pub(crate) fn pop_frame(&mut self) -> Result<usize, InternalRuntimeError> {
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

    pub(crate) fn push_frame(&mut self, ip: usize, arguments: Vec<Value>, stack: Option<Stack>) {
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
            let InternalValue::Return{ ghost: Some(mut stack), .. } = self.cactus.at(self.len() - self.frame).unwrap() else { panic!() };
            return stack.replace_at_local(index, value);
        }
        self.cactus
            .replace_at(register, InternalValue::Value(value))
            .map_err(|_| InternalRuntimeError::ExpectedValue)
            .and_then(InternalValue::try_into_value)
    }

    pub(crate) fn pop_n(&mut self, arity: usize) -> Result<Vec<Value>, InternalRuntimeError> {
        let internal_values = self
            .cactus
            .detach_at(arity)
            .ok_or(InternalRuntimeError::ExpectedValue)?;
        internal_values
            .into_iter()
            .map(InternalValue::try_into_value)
            .collect()
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
