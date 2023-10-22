use std::cmp::Ordering;

use num::ToPrimitive;

use super::error::{ErrorKind, InternalRuntimeError};
use super::program::ProgramReader;
use super::stack::InternalValue;
use super::{Error, Stack};
use crate::atom::AtomInterner;
use crate::callable::{Continuation, Procedure};
use crate::runtime::callable::{Callable, CallableKind};
use crate::{Atom, Instruction, Number, Offset, ReferentialEq, Struct, StructuralEq, Tuple, Value};

pub(super) enum Step<E> {
    End,
    Continue,
    #[allow(dead_code)]
    Suspend,
    Spawn(E),
    Exit(Value),
}

/// Represents a currently active execution of the Trilogy VM on a program.
///
/// This is received from within
pub struct Execution<'a> {
    pub(super) atom_interner: AtomInterner,
    program: ProgramReader<'a>,
    ip: Offset,
    stack: Stack,
    registers: Vec<Value>,
    stack_stack: Vec<(Offset, Stack)>,
}

impl<'a> Execution<'a> {
    pub(super) fn new(
        atom_interner: AtomInterner,
        program: ProgramReader<'a>,
        registers: Vec<Value>,
    ) -> Self {
        Self {
            atom_interner: atom_interner.clone(),
            ip: program.entrypoint(),
            program,
            stack: Stack::default(),
            stack_stack: vec![],
            registers,
        }
    }

    /// Create an atom in the context of the VM that this Execution belongs to..
    pub fn atom(&self, atom: &str) -> Atom {
        self.atom_interner.intern(atom)
    }

    /// Create an anonymous atom, that can never be recreated.
    pub fn atom_anon(&self, label: &str) -> Atom {
        Atom::new_unique(label.to_owned())
    }

    fn branch(&mut self) -> Self {
        let branch = self.stack.branch();
        Self {
            atom_interner: self.atom_interner.clone(),
            program: self.program.clone(),
            ip: self.ip,
            stack: branch,
            stack_stack: vec![],
            registers: self.registers.clone(),
        }
    }

    fn current_continuation(&mut self) -> Continuation {
        Continuation::new(self.ip, self.stack.branch())
    }

    fn current_closure(&mut self) -> Procedure {
        Procedure::new_closure(self.ip, self.stack.branch())
    }

    fn call_continuation(
        &mut self,
        continuation: Continuation,
        args: Vec<InternalValue>,
    ) -> Result<(), Error> {
        let running_stack = continuation.stack();
        let paused_stack = std::mem::replace(&mut self.stack, running_stack);
        self.stack_stack.push((self.ip, paused_stack));
        self.stack.push_many(args);
        self.ip = continuation.ip();
        Ok(())
    }

    fn become_continuation(&mut self, continuation: Continuation, args: Vec<InternalValue>) {
        self.stack = continuation.stack();
        self.stack.push_many(args);
        self.ip = continuation.ip();
    }

    fn reset_continuation(&mut self) -> Result<(), Error> {
        let (ip, running_stack) = self.stack_stack.pop().ok_or_else(|| {
            self.error(ErrorKind::InternalRuntimeError(
                InternalRuntimeError::ExpectedStack,
            ))
        })?;
        self.ip = ip;
        self.stack = running_stack;
        Ok(())
    }

    fn call_procedure(&mut self, procedure: Procedure, args: Vec<InternalValue>) {
        self.stack.push_frame(self.ip, args, procedure.stack());
        self.ip = procedure.ip();
    }

    fn become_procedure(
        &mut self,
        procedure: Procedure,
        args: Vec<InternalValue>,
    ) -> Result<(), Error> {
        let ip = self.stack.pop_frame().map_err(|k| self.error(k))?;
        self.stack.push_frame(ip, args, procedure.stack());
        self.ip = procedure.ip();
        Ok(())
    }

    pub(super) fn error<K>(&self, kind: K) -> Error
    where
        ErrorKind: From<K>,
    {
        Error {
            ip: self.ip,
            stack_dump: self.stack.clone(),
            kind: kind.into(),
        }
    }

    fn read_local(&self, index: usize) -> Result<Value, Error> {
        self.stack.at_local(index).map_err(|k| self.error(k))
    }

    fn is_set_local(&self, index: usize) -> Result<bool, Error> {
        self.stack.is_set_local(index).map_err(|k| self.error(k))
    }

    fn set_local(&mut self, index: usize, value: Value) -> Result<Option<Value>, Error> {
        self.stack
            .set_local(index, value)
            .map_err(|k| self.error(k))
    }

    fn push_unset(&mut self) {
        self.stack.push_unset();
    }

    fn unset_local(&mut self, index: usize) -> Result<Option<Value>, Error> {
        self.stack.unset_local(index).map_err(|k| self.error(k))
    }

    fn init_local(&mut self, index: usize, value: Value) -> Result<bool, Error> {
        self.stack
            .init_local(index, value)
            .map_err(|k| self.error(k))
    }

    fn stack_peek(&self) -> Result<Value, Error> {
        self.stack.at(0).map_err(|k| self.error(k))
    }

    fn stack_peek_raw(&self) -> Result<InternalValue, Error> {
        self.stack.at_raw(0).map_err(|k| self.error(k))
    }

    fn stack_pop(&mut self) -> Result<Value, Error> {
        self.stack_discard().and_then(|v| {
            v.ok_or_else(|| self.error(InternalRuntimeError::ExpectedValue("empty stack")))
        })
    }

    fn stack_discard(&mut self) -> Result<Option<Value>, Error> {
        self.stack.pop().map_err(|k| self.error(k))
    }

    fn stack_slide(&mut self, n: usize) -> Result<(), Error> {
        let top = self.stack_pop()?;
        let slide = self.stack.pop_n(n).map_err(|k| self.error(k))?;
        self.stack.push(top);
        self.stack.push_many(slide);
        Ok(())
    }

    fn stack_push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn call(&mut self, arity: usize) -> Result<(), Error> {
        let arguments = self.stack.pop_n(arity).map_err(|k| self.error(k))?;
        let callable = self.stack.pop().map_err(|k| self.error(k))?;
        match callable {
            Some(Value::Callable(Callable(CallableKind::Continuation(continuation)))) => {
                self.call_continuation(continuation, arguments)?;
            }
            Some(Value::Callable(Callable(CallableKind::Procedure(procedure)))) => {
                self.call_procedure(procedure, arguments);
            }
            Some(Value::Callable(Callable(CallableKind::Native(native)))) => {
                let ret_val = native.call(
                    self.branch(),
                    arguments
                        .into_iter()
                        .map(|val| val.try_into_value())
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|k| self.error(k))?,
                );
                self.stack.push(ret_val);
            }
            _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
        }
        Ok(())
    }

    fn r#become(&mut self, arity: usize) -> Result<(), Error> {
        let arguments = self.stack.pop_n(arity).map_err(|k| self.error(k))?;
        let callable = self.stack.pop().map_err(|k| self.error(k))?;
        match callable {
            Some(Value::Callable(Callable(CallableKind::Continuation(continuation)))) => {
                self.become_continuation(continuation, arguments);
            }
            Some(Value::Callable(Callable(CallableKind::Procedure(procedure)))) => {
                self.become_procedure(procedure, arguments)?;
            }
            Some(Value::Callable(Callable(CallableKind::Native(native)))) => {
                let ret_val = native.call(
                    self.branch(),
                    arguments
                        .into_iter()
                        .map(|val| val.try_into_value())
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|k| self.error(k))?,
                );
                self.r#return()?;
                self.stack.push(ret_val);
            }
            _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
        }
        Ok(())
    }

    fn r#return(&mut self) -> Result<(), Error> {
        let ip = self.stack.pop_frame().map_err(|k| self.error(k))?;
        self.ip = ip;
        Ok(())
    }

    pub(super) fn step(&mut self) -> Result<Step<Self>, Error> {
        let instruction = self.program.read_instruction(self.ip);
        self.ip += instruction.byte_len() as u32;
        match instruction {
            Instruction::Const(value) => {
                self.stack_push(value.structural_clone());
            }
            Instruction::LoadLocal(offset) => {
                self.stack_push(self.read_local(offset as usize)?);
            }
            Instruction::Variable => {
                self.push_unset();
            }
            Instruction::SetLocal(offset) => {
                let value = self.stack_pop()?;
                self.set_local(offset as usize, value)?;
            }
            Instruction::UnsetLocal(offset) => {
                self.unset_local(offset as usize)?;
            }
            Instruction::InitLocal(offset) => {
                let value = self.stack_pop()?;
                let did_set = self.init_local(offset as usize, value)?;
                self.stack_push(did_set.into());
            }
            Instruction::IsSetLocal(offset) => {
                let is_set = self.is_set_local(offset as usize)?;
                self.stack_push(is_set.into());
            }
            Instruction::LoadRegister(offset) => {
                if offset as usize >= self.registers.len() {
                    return Err(self.error(InternalRuntimeError::InvalidRegister(offset)));
                }
                self.stack_push(self.registers[offset as usize].clone());
            }
            Instruction::SetRegister(offset) => {
                let value = self.stack_pop()?;
                if offset as usize >= self.registers.len() {
                    return Err(self.error(InternalRuntimeError::InvalidRegister(offset)));
                }
                self.registers[offset as usize] = value;
            }
            Instruction::Pop => {
                self.stack_discard()?;
            }
            Instruction::Swap => {
                self.stack_slide(1)?;
            }
            Instruction::Slide(offset) => {
                self.stack_slide(offset as usize)?;
            }
            Instruction::Copy => {
                let value = self.stack_peek()?;
                self.stack_push(value);
            }
            Instruction::Clone => {
                let value = self.stack_pop()?;
                self.stack_push(value.structural_clone());
            }
            Instruction::TypeOf => {
                let value = self.stack_pop()?;
                match value {
                    Value::Unit => self.stack_push("unit".into()),
                    Value::Number(..) => self.stack_push("number".into()),
                    Value::Bits(..) => self.stack_push("bits".into()),
                    Value::Bool(..) => self.stack_push("boolean".into()),
                    Value::String(..) => self.stack_push("string".into()),
                    Value::Char(..) => self.stack_push("character".into()),
                    Value::Tuple(..) => self.stack_push("tuple".into()),
                    Value::Array(..) => self.stack_push("array".into()),
                    Value::Set(..) => self.stack_push("set".into()),
                    Value::Record(..) => self.stack_push("record".into()),
                    Value::Atom(..) => self.stack_push("atom".into()),
                    Value::Struct(..) => self.stack_push("struct".into()),
                    Value::Callable(..) => self.stack_push("callable".into()),
                }
            }
            Instruction::Add => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match lhs + rhs {
                    Ok(val) => self.stack_push(val),
                    Err(..) => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
            }
            Instruction::Subtract => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match lhs - rhs {
                    Ok(val) => self.stack_push(val),
                    Err(..) => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
            }
            Instruction::Multiply => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match lhs * rhs {
                    Ok(val) => self.stack_push(val),
                    Err(..) => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
            }
            Instruction::Divide => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match lhs / rhs {
                    Ok(val) => self.stack_push(val),
                    Err(..) => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
            }
            Instruction::Remainder => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match lhs % rhs {
                    Ok(val) => self.stack_push(val),
                    Err(..) => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
            }
            Instruction::IntDivide => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match lhs / rhs {
                    Ok(Value::Number(val)) => {
                        self.stack_push(Number::from(val.as_complex().re.floor()).into());
                    }
                    _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
            }
            Instruction::Power => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match (lhs, rhs) {
                    (Value::Number(lhs), Value::Number(rhs)) => {
                        self.stack_push(Value::Number(lhs.pow(&rhs)));
                    }
                    _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
            }
            Instruction::Negate => {
                let val = self.stack_pop()?;
                match -val {
                    Ok(val) => self.stack_push(val),
                    Err(..) => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
            }
            Instruction::Glue => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match (lhs, rhs) {
                    (Value::String(lhs), Value::String(rhs)) => {
                        self.stack_push(Value::String(lhs + &rhs))
                    }
                    (Value::Array(lhs), Value::Array(rhs)) => {
                        lhs.append(&rhs);
                        self.stack_push(Value::Array(lhs));
                    }
                    (Value::Set(lhs), Value::Set(rhs)) => {
                        lhs.union(&rhs);
                        self.stack_push(Value::Set(lhs));
                    }
                    (Value::Record(lhs), Value::Record(rhs)) => {
                        lhs.union(&rhs);
                        self.stack_push(Value::Record(lhs));
                    }
                    _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
            }
            Instruction::Skip => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                let count = match rhs {
                    Value::Number(number) if number.is_uinteger() => number
                        .as_uinteger()
                        .unwrap()
                        .to_usize()
                        .ok_or(self.error(ErrorKind::RuntimeTypeError))?,
                    _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
                };
                match lhs {
                    Value::String(lhs) => {
                        self.stack_push(Value::String(lhs.chars().skip(count).collect()))
                    }
                    Value::Array(lhs) => {
                        self.stack_push(Value::Array(lhs.range(count..).to_owned()))
                    }
                    _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
            }
            Instruction::Take => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                let count = match rhs {
                    Value::Number(number) if number.is_uinteger() => number
                        .as_uinteger()
                        .unwrap()
                        .to_usize()
                        .ok_or(self.error(ErrorKind::RuntimeTypeError))?,
                    _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
                };
                match lhs {
                    Value::String(lhs) => {
                        self.stack_push(Value::String(lhs.chars().take(count).collect()));
                    }
                    Value::Array(lhs) => {
                        self.stack_push(Value::Array(lhs.range(..count).to_owned()));
                    }
                    _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
            }
            Instruction::Access => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match (lhs, rhs) {
                    (Value::Record(record), rhs) => match record.get(&rhs) {
                        Some(value) => self.stack_push(value),
                        None => return Err(self.error(ErrorKind::RuntimeTypeError)),
                    },
                    (Value::String(lhs), Value::Number(rhs)) => {
                        let ch = rhs
                            .as_uinteger()
                            .and_then(|index| index.to_usize())
                            .and_then(|index| lhs.chars().nth(index));
                        match ch {
                            Some(ch) => self.stack_push(Value::Char(ch)),
                            None => return Err(self.error(ErrorKind::RuntimeTypeError)),
                        }
                    }
                    (Value::Bits(lhs), Value::Number(rhs)) => {
                        let val = rhs
                            .as_uinteger()
                            .and_then(|index| index.to_usize())
                            .and_then(|index| lhs.get(index));
                        match val {
                            Some(val) => self.stack_push(Value::Bool(val)),
                            None => return Err(self.error(ErrorKind::RuntimeTypeError)),
                        }
                    }
                    (Value::Array(lhs), Value::Number(rhs)) => {
                        let val = rhs
                            .as_uinteger()
                            .and_then(|index| index.to_usize())
                            .and_then(|index| lhs.get(index));
                        match val {
                            Some(val) => self.stack_push(val),
                            None => return Err(self.error(ErrorKind::RuntimeTypeError)),
                        }
                    }
                    _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
            }
            Instruction::Assign => {
                let value = self.stack_pop()?;
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match (&lhs, rhs, value) {
                    (Value::Record(record), rhs, value) => {
                        record.insert(rhs, value);
                    }
                    (Value::Array(lhs), Value::Number(rhs), value) => {
                        let index = rhs.as_uinteger().and_then(|index| index.to_usize());
                        match index {
                            Some(index) => lhs.set(index, value),
                            None => return Err(self.error(ErrorKind::RuntimeTypeError)),
                        }
                    }
                    _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
                self.stack_push(lhs);
            }
            Instruction::Length => {
                let value = self.stack_pop()?;
                match value {
                    Value::Array(arr) => self.stack_push(Value::from(arr.len())),
                    Value::Record(record) => self.stack_push(Value::from(record.len())),
                    Value::Set(set) => self.stack_push(Value::from(set.len())),
                    Value::String(string) => self.stack_push(Value::from(string.len())),
                    _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
            }
            Instruction::Insert => {
                let value = self.stack_pop()?;
                let collection = self.stack_pop()?;
                match &collection {
                    Value::Array(arr) => {
                        arr.push(value);
                    }
                    Value::Set(set) => {
                        set.insert(value);
                    }
                    _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
                self.stack_push(collection);
            }
            Instruction::Delete => {
                let key = self.stack_pop()?;
                let value = self.stack_pop()?;
                match &value {
                    Value::Record(record) => {
                        record.remove(&key);
                    }
                    Value::Set(set) => {
                        set.remove(&key);
                    }
                    Value::Array(arr) => {
                        let Value::Number(number) = key else {
                            return Err(self.error(ErrorKind::RuntimeTypeError));
                        };
                        let Some(index) = number.as_uinteger().and_then(|index| index.to_usize())
                        else {
                            return Err(self.error(ErrorKind::RuntimeTypeError));
                        };
                        arr.remove(index);
                    }
                    _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
                self.stack_push(value);
            }
            Instruction::Contains => {
                let key = self.stack_pop()?;
                let value = self.stack_pop()?;
                match value {
                    Value::Record(record) => {
                        self.stack_push(record.contains_key(&key).into());
                    }
                    Value::Set(set) => {
                        self.stack_push(set.has(&key).into());
                    }
                    Value::Array(arr) => {
                        self.stack_push(arr.contains(&key).into());
                    }
                    _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
            }
            Instruction::Entries => {
                let collection = self.stack_pop()?;
                match collection {
                    Value::Record(record) => {
                        self.stack_push(
                            record
                                .into_iter()
                                .map(Into::into)
                                .collect::<Vec<Value>>()
                                .into(),
                        );
                    }
                    Value::Set(set) => {
                        self.stack_push(set.into_iter().collect::<Vec<Value>>().into());
                    }
                    value @ Value::Array(..) => {
                        self.stack_push(value);
                    }
                    _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
            }
            Instruction::Not => {
                let val = self.stack_pop()?;
                match val {
                    Value::Bool(val) => self.stack_push(Value::Bool(!val)),
                    _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
            }
            Instruction::And => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match (lhs, rhs) {
                    (Value::Bool(lhs), Value::Bool(rhs)) => {
                        self.stack_push(Value::Bool(lhs && rhs))
                    }
                    _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
            }
            Instruction::Or => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match (lhs, rhs) {
                    (Value::Bool(lhs), Value::Bool(rhs)) => {
                        self.stack_push(Value::Bool(lhs || rhs))
                    }
                    _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
            }
            Instruction::BitwiseAnd => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match lhs & rhs {
                    Ok(val) => self.stack_push(val),
                    _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
            }
            Instruction::BitwiseOr => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match lhs | rhs {
                    Ok(val) => self.stack_push(val),
                    _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
            }
            Instruction::BitwiseXor => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match lhs ^ rhs {
                    Ok(val) => self.stack_push(val),
                    _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
            }
            Instruction::BitwiseNeg => {
                let val = self.stack_pop()?;
                match val {
                    Value::Bits(val) => self.stack_push(Value::Bits(!val)),
                    _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
            }
            Instruction::LeftShift => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match lhs << rhs {
                    Ok(val) => self.stack_push(val),
                    _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
            }
            Instruction::RightShift => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match lhs >> rhs {
                    Ok(val) => self.stack_push(val),
                    _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
            }
            Instruction::Cons => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                self.stack_push(Value::Tuple(Tuple::new(lhs, rhs)));
            }
            Instruction::Uncons => {
                let (first, second) = self.stack_pop().and_then(|val| match val {
                    Value::Tuple(tuple) => Ok(tuple.uncons()),
                    _ => Err(self.error(ErrorKind::RuntimeTypeError)),
                })?;
                self.stack_push(first);
                self.stack_push(second);
            }
            Instruction::First => {
                let first = self.stack_pop().and_then(|val| match val {
                    Value::Tuple(tuple) => Ok(tuple.into_first()),
                    _ => Err(self.error(ErrorKind::RuntimeTypeError)),
                })?;
                self.stack_push(first);
            }
            Instruction::Second => {
                let first = self.stack_pop().and_then(|val| match val {
                    Value::Tuple(tuple) => Ok(tuple.into_second()),
                    _ => Err(self.error(ErrorKind::RuntimeTypeError)),
                })?;
                self.stack_push(first);
            }
            Instruction::Construct => {
                let atom = self.stack_pop().and_then(|val| match val {
                    Value::Atom(atom) => Ok(atom),
                    _ => Err(self.error(ErrorKind::RuntimeTypeError)),
                })?;
                let value = self.stack_pop()?;
                self.stack_push(Value::Struct(Struct::new(atom, value)));
            }
            Instruction::Destruct => {
                let (atom, value) = self.stack_pop().and_then(|val| match val {
                    Value::Struct(val) => Ok(val.destruct()),
                    _ => Err(self.error(ErrorKind::RuntimeTypeError)),
                })?;
                self.stack_push(value);
                self.stack_push(atom.into());
            }
            Instruction::Leq => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                let cmp = match lhs.partial_cmp(&rhs) {
                    Some(Ordering::Less | Ordering::Equal) => Value::Bool(true),
                    Some(Ordering::Greater) => Value::Bool(false),
                    None => Value::Unit,
                };
                self.stack_push(cmp);
            }
            Instruction::Lt => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                let cmp = match lhs.partial_cmp(&rhs) {
                    Some(Ordering::Less) => Value::Bool(true),
                    Some(Ordering::Greater) | Some(Ordering::Equal) => Value::Bool(false),
                    None => Value::Unit,
                };
                self.stack_push(cmp);
            }
            Instruction::Geq => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                let cmp = match lhs.partial_cmp(&rhs) {
                    Some(Ordering::Less) => Value::Bool(false),
                    Some(Ordering::Greater) | Some(Ordering::Equal) => Value::Bool(true),
                    None => Value::Unit,
                };
                self.stack_push(cmp);
            }
            Instruction::Gt => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                let cmp = match lhs.partial_cmp(&rhs) {
                    Some(Ordering::Less) | Some(Ordering::Equal) => Value::Bool(false),
                    Some(Ordering::Greater) => Value::Bool(true),
                    None => Value::Unit,
                };
                self.stack_push(cmp);
            }
            Instruction::RefEq => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                self.stack_push(Value::Bool(ReferentialEq::eq(&lhs, &rhs)));
            }
            Instruction::ValEq => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                self.stack_push(Value::Bool(StructuralEq::eq(&lhs, &rhs)));
            }
            Instruction::RefNeq => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                self.stack_push(Value::Bool(!ReferentialEq::eq(&lhs, &rhs)));
            }
            Instruction::ValNeq => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                self.stack_push(Value::Bool(!StructuralEq::eq(&lhs, &rhs)));
            }
            Instruction::Call(arity) => {
                self.call(arity as usize)?;
            }
            Instruction::Become(arity) => {
                self.r#become(arity as usize)?;
            }
            Instruction::Return => {
                let return_value = self.stack_pop()?;
                self.r#return()?;
                self.stack_push(return_value);
            }
            Instruction::Close(offset) => {
                let closure = self.current_closure();
                self.stack_push(Value::from(closure));
                self.ip = offset;
            }
            Instruction::Shift(offset) => {
                let continuation = self.current_continuation();
                self.stack_push(Value::from(continuation));
                self.ip = offset;
            }
            Instruction::Reset => {
                let return_value = self.stack_pop()?;
                self.reset_continuation()?;
                self.stack_push(return_value);
            }
            Instruction::Jump(offset) => {
                self.ip = offset;
            }
            Instruction::CondJump(offset) => {
                let cond = self.stack_pop()?;
                match cond {
                    Value::Bool(false) => self.ip = offset,
                    Value::Bool(true) => {}
                    _ => return Err(self.error(ErrorKind::RuntimeTypeError)),
                }
            }
            Instruction::Branch => {
                // A branch requires two values on the stack; the two branches get the
                // different values, respectively.
                let right = self.stack_pop()?;
                let left = self.stack_pop()?;
                let mut branch = self.branch();
                self.stack_push(left);
                branch.stack_push(right);
                return Ok(Step::Spawn(branch));
            }
            Instruction::Fizzle => return Ok(Step::End),
            Instruction::Exit => {
                // When run in embedded mode, the exit value can be any value. The
                // interpreter binary can decide how to handle that exit value when
                // passing off to the OS.
                let value = self.stack_pop()?;
                return Ok(Step::Exit(value));
            }
            Instruction::Chunk(locator) => {
                let value = self
                    .program
                    .locate(locator)
                    .map_err(|er| self.error(ErrorKind::InvalidBytecode(er)))?;
                self.stack_push(value);
            }
            Instruction::Debug => {
                let val = self.stack_peek_raw()?;
                eprintln!("{}", val);
            }
        }
        Ok(Step::Continue)
    }
}
