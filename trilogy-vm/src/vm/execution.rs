use super::error::{ErrorKind, InternalRuntimeError};
use super::program_reader::ProgramReader;
use super::stack::{InternalValue, Stack};
use super::Error;
use crate::atom::AtomInterner;
use crate::callable::{Closure, Continuation};
use crate::runtime::callable::{Callable, CallableKind};
use crate::{Atom, Instruction, Number, Offset, ReferentialEq, Struct, StructuralEq, Tuple, Value};
use num::ToPrimitive;
use std::cmp::Ordering;
use std::fmt::{self, Debug};
use std::sync::{Arc, Mutex};

pub(super) enum Step<E> {
    End,
    Continue,
    #[allow(dead_code)]
    Suspend,
    Spawn(E),
    Exit(Value),
}

#[allow(clippy::type_complexity)]
#[derive(Clone)]
pub(super) struct Callback(
    Arc<Mutex<dyn FnMut(&mut Execution, Value) -> Result<(), Error> + Sync + Send + 'static>>,
);

#[derive(Clone)]
pub(super) enum Cont {
    Offset(Offset),
    Callback(Callback),
}

impl Debug for Cont {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Offset(ip) => write!(f, "{ip}"),
            Self::Callback(..) => write!(f, "rust"),
        }
    }
}

impl<F> From<F> for Cont
where
    F: FnMut(&mut Execution, Value) -> Result<(), Error> + Sync + Send + 'static,
{
    fn from(value: F) -> Self {
        Cont::Callback(Callback(Arc::new(Mutex::new(value))))
    }
}

impl From<Offset> for Cont {
    fn from(value: Offset) -> Self {
        Cont::Offset(value)
    }
}

/// Represents a currently active execution of the Trilogy VM on a program.
///
/// Each execution has its own stack, continuations, and registers. Any invocation
/// of a Trilogy program may have more than one execution, as an execution can be
/// split at any point by using the `BRANCH` instruction.
///
/// Native functions, in their raw form, are provided with an execution, allowing them
/// to call back into the Trilogy runtime to emulate features that pure programs would
/// have access to. In practice there should be no need to work with an Execution directly
/// as the implementation of the language frontend and FFI layer should safely wrap the
/// Execution's low level operations in a more runtime appropriate way.
pub struct Execution<'a> {
    atom_interner: AtomInterner,
    program: ProgramReader<'a>,
    error_ip: Offset,
    ip: Offset,
    stack: Stack,
    registers: Vec<Value>,
}

impl<'a> Execution<'a> {
    pub(super) fn new(
        atom_interner: AtomInterner,
        program: ProgramReader<'a>,
        registers: Vec<Value>,
    ) -> Self {
        Self {
            atom_interner: atom_interner.clone(),
            error_ip: 0,
            ip: program.entrypoint(),
            program,
            stack: Stack::default(),
            registers,
        }
    }

    /// Create an atom in the context of the VM that this Execution belongs to.
    ///
    /// See [`Atom`][] for more details.
    pub fn atom(&self, atom: &str) -> Atom {
        self.atom_interner.intern(atom)
    }

    /// Create an anonymous atom, that can never be recreated.
    ///
    /// See [`Atom`][] for more details.
    pub fn atom_anon(&self, label: &str) -> Atom {
        Atom::new_unique(label.to_owned())
    }

    /// Call back into the Trilogy runtime from outside by invoking another Trilogy Callable
    /// value.
    ///
    /// Due to Trilogy control flow being more powerful than Rust control flow, instead of this
    /// call returning any values, it instead calls a provided callback with the return value
    /// of the call. If the call returns more or less than once, the callback will be called as
    /// many times.
    ///
    /// Be careful to call with the right number of arguments, as this cannot be checked statically.
    /// If a callable is called with incorrect arity, strange things may occur.
    pub fn callback<
        F: FnMut(&mut Execution, Value) -> Result<(), Error> + Sync + Send + 'static,
    >(
        &mut self,
        callable: Value,
        arguments: Vec<Value>,
        callback: F,
    ) -> Result<(), Error> {
        match callable {
            Value::Callable(Callable(CallableKind::Continuation(continuation))) => {
                self.stack = continuation.stack();
                self.stack
                    .push_many(arguments.into_iter().map(InternalValue::Value).collect());
                self.ip = continuation.ip();
            }
            Value::Callable(Callable(CallableKind::Procedure(procedure))) => {
                self.stack.push_frame(
                    callback,
                    arguments.into_iter().map(InternalValue::Value).collect(),
                    None,
                );
                self.ip = procedure.ip();
            }
            Value::Callable(Callable(CallableKind::Closure(closure))) => {
                self.stack.push_frame(
                    callback,
                    arguments.into_iter().map(InternalValue::Value).collect(),
                    Some(closure.stack().clone()),
                );
                self.ip = closure.ip();
            }
            Value::Callable(Callable(CallableKind::Native(native))) => {
                self.stack.push_frame(callback, vec![], None);
                native.call(self, arguments)?;
            }
            _ => return Err(self.error(InternalRuntimeError::TypeError)),
        }
        Ok(())
    }

    /// Creates a callable value that corresponds to a specific label in the Program.
    ///
    /// This is a very low-level operation, which is treated as part of the code-generation
    /// process. It is up to the code generator to ensure that all labels that might be
    /// referenced in this way are in fact pre-compiled into the bytecode.
    pub fn procedure(&self, label: &str) -> Result<Value, Error> {
        self.program.procedure(label).map_err(|k| self.error(k))
    }

    fn branch(&mut self) -> Self {
        let branch = self.stack.branch();
        Self {
            atom_interner: self.atom_interner.clone(),
            program: self.program.clone(),
            error_ip: self.error_ip,
            ip: self.ip,
            stack: branch,
            registers: self.registers.clone(),
        }
    }

    pub fn error<K>(&self, kind: K) -> Error
    where
        ErrorKind: From<K>,
    {
        Error {
            ip: self.error_ip,
            stack_trace: self.stack.trace(&self.program, self.error_ip),
            stack_dump: self.stack.clone(),
            kind: kind.into(),
        }
    }

    fn stack_pop(&mut self) -> Result<Value, Error> {
        self.stack.pop().map_err(|k| self.error(k)).and_then(|v| {
            v.ok_or_else(|| self.error(InternalRuntimeError::ExpectedValue("empty stack")))
        })
    }

    /// Return a value from the current procedure.
    ///
    /// The returned value will become the result of the `CALL` instruction for
    /// this call.
    ///
    /// When used in a native function, return must only be called once. It is impossible
    /// to return more than once from a native function, despite return being provided as
    /// a function instead of using the functions actual return value.
    pub fn r#return(&mut self, return_value: Value) -> Result<(), Error> {
        let cont = self.stack.pop_frame().map_err(|k| self.error(k))?;
        match cont {
            Cont::Offset(ip) => {
                self.ip = ip;
                self.stack.push(return_value);
                Ok(())
            }
            Cont::Callback(cb) => {
                let mut callback = cb.0.lock().unwrap();
                callback(self, return_value)
            }
        }
    }

    fn call_internal(
        &mut self,
        callable: Value,
        arguments: Vec<InternalValue>,
    ) -> Result<(), Error> {
        match callable {
            Value::Callable(Callable(CallableKind::Continuation(continuation))) => {
                self.stack = continuation.stack();
                self.stack.push_many(arguments);
                self.ip = continuation.ip();
            }
            Value::Callable(Callable(CallableKind::Procedure(procedure))) => {
                self.stack.push_frame(self.ip, arguments, None);
                self.ip = procedure.ip();
            }
            Value::Callable(Callable(CallableKind::Closure(closure))) => {
                self.stack
                    .push_frame(self.ip, arguments, Some(closure.stack().clone()));
                self.ip = closure.ip();
            }
            Value::Callable(Callable(CallableKind::Native(native))) => {
                self.stack.push_frame(self.ip, vec![], None);
                native.call(
                    self,
                    arguments
                        .into_iter()
                        .map(|val| val.try_into_value())
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|k| self.error(k))?,
                )?;
            }
            _ => return Err(self.error(InternalRuntimeError::TypeError)),
        }
        Ok(())
    }

    fn r#become(&mut self, arity: usize) -> Result<(), Error> {
        let arguments = self.stack.pop_n(arity).map_err(|k| self.error(k))?;
        let callable = self.stack.pop().map_err(|k| self.error(k))?;
        match callable {
            Some(Value::Callable(Callable(CallableKind::Continuation(continuation)))) => {
                self.stack = continuation.stack();
                self.stack.push_many(arguments);
                self.ip = continuation.ip();
            }
            Some(Value::Callable(Callable(CallableKind::Procedure(procedure)))) => {
                let ip = self.stack.pop_frame().map_err(|k| self.error(k))?;
                self.stack.push_frame(ip, arguments, None);
                self.ip = procedure.ip();
            }
            Some(Value::Callable(Callable(CallableKind::Closure(closure)))) => {
                let ip = self.stack.pop_frame().map_err(|k| self.error(k))?;
                self.stack
                    .push_frame(ip, arguments, Some(closure.stack().clone()));
                self.ip = closure.ip();
            }
            Some(Value::Callable(Callable(CallableKind::Native(native)))) => {
                let ip = self.stack.pop_frame().map_err(|k| self.error(k))?;
                self.stack.push_frame(ip, vec![], None);
                native.call(
                    self,
                    arguments
                        .into_iter()
                        .map(|val| val.try_into_value())
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|k| self.error(k))?,
                )?;
            }
            _ => return Err(self.error(InternalRuntimeError::TypeError)),
        }
        Ok(())
    }

    pub(super) fn step(&mut self) -> Result<Step<Self>, Error> {
        let instruction = self.program.read_instruction(self.ip);
        self.error_ip = self.ip;
        self.ip += instruction.byte_len() as u32;
        self.eval(instruction)
    }

    fn eval(&mut self, instruction: Instruction) -> Result<Step<Self>, Error> {
        match instruction {
            Instruction::Const(value) => {
                self.stack.push(value.structural_clone());
            }
            Instruction::LoadLocal(offset) => {
                let value = self
                    .stack
                    .at_local(offset as usize)
                    .map_err(|k| self.error(k))?;
                self.stack.push(value);
            }
            Instruction::Variable => {
                self.stack.push_unset();
            }
            Instruction::SetLocal(offset) => {
                let value = self.stack_pop()?;
                self.stack
                    .set_local(offset as usize, value)
                    .map_err(|k| self.error(k))?;
            }
            Instruction::UnsetLocal(offset) => {
                self.stack
                    .unset_local(offset as usize)
                    .map_err(|k| self.error(k))?;
            }
            Instruction::InitLocal(offset) => {
                let value = self.stack_pop()?;
                let did_set = self
                    .stack
                    .init_local(offset as usize, value)
                    .map_err(|k| self.error(k))?;
                self.stack.push(did_set);
            }
            Instruction::IsSetLocal(offset) => {
                let is_set = self
                    .stack
                    .is_set_local(offset as usize)
                    .map_err(|k| self.error(k))?;
                self.stack.push(is_set);
            }
            Instruction::LoadRegister(offset) => {
                if offset as usize >= self.registers.len() {
                    return Err(self.error(InternalRuntimeError::InvalidRegister(offset)));
                }
                self.stack.push(self.registers[offset as usize].clone());
            }
            Instruction::SetRegister(offset) => {
                let value = self.stack_pop()?;
                if offset as usize >= self.registers.len() {
                    return Err(self.error(InternalRuntimeError::InvalidRegister(offset)));
                }
                self.registers[offset as usize] = value;
            }
            Instruction::Pop => {
                self.stack.pop().map_err(|k| self.error(k))?;
            }
            Instruction::Swap => {
                self.stack.slide(1).map_err(|k| self.error(k))?;
            }
            Instruction::Slide(offset) => {
                self.stack
                    .slide(offset as usize)
                    .map_err(|k| self.error(k))?;
            }
            Instruction::Copy => {
                let value = self.stack.at(0).map_err(|k| self.error(k))?;
                self.stack.push(value);
            }
            Instruction::Clone => {
                let value = self.stack_pop()?;
                self.stack.push(value.shallow_clone());
            }
            Instruction::DeepClone => {
                let value = self.stack_pop()?;
                self.stack.push(value.shallow_clone());
            }
            Instruction::TypeOf => {
                let value = self.stack_pop()?;
                match value {
                    Value::Unit => self.stack.push(self.atom("unit")),
                    Value::Number(..) => self.stack.push(self.atom("number")),
                    Value::Bits(..) => self.stack.push(self.atom("bits")),
                    Value::Bool(..) => self.stack.push(self.atom("boolean")),
                    Value::String(..) => self.stack.push(self.atom("string")),
                    Value::Char(..) => self.stack.push(self.atom("character")),
                    Value::Tuple(..) => self.stack.push(self.atom("tuple")),
                    Value::Array(..) => self.stack.push(self.atom("array")),
                    Value::Set(..) => self.stack.push(self.atom("set")),
                    Value::Record(..) => self.stack.push(self.atom("record")),
                    Value::Atom(..) => self.stack.push(self.atom("atom")),
                    Value::Struct(..) => self.stack.push(self.atom("struct")),
                    Value::Callable(..) => self.stack.push(self.atom("callable")),
                }
            }
            Instruction::Add => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match lhs + rhs {
                    Ok(val) => self.stack.push(val),
                    Err(..) => return Err(self.error(InternalRuntimeError::TypeError)),
                }
            }
            Instruction::Subtract => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match lhs - rhs {
                    Ok(val) => self.stack.push(val),
                    Err(..) => return Err(self.error(InternalRuntimeError::TypeError)),
                }
            }
            Instruction::Multiply => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match lhs * rhs {
                    Ok(val) => self.stack.push(val),
                    Err(..) => return Err(self.error(InternalRuntimeError::TypeError)),
                }
            }
            Instruction::Divide => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match lhs / rhs {
                    Ok(val) => self.stack.push(val),
                    Err(..) => return Err(self.error(InternalRuntimeError::TypeError)),
                }
            }
            Instruction::Remainder => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match lhs % rhs {
                    Ok(val) => self.stack.push(val),
                    Err(..) => return Err(self.error(InternalRuntimeError::TypeError)),
                }
            }
            Instruction::IntDivide => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match lhs / rhs {
                    Ok(Value::Number(val)) => {
                        self.stack.push(Number::from(val.as_complex().re.floor()));
                    }
                    _ => return Err(self.error(InternalRuntimeError::TypeError)),
                }
            }
            Instruction::Power => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match (lhs, rhs) {
                    (Value::Number(lhs), Value::Number(rhs)) => {
                        self.stack.push(Value::Number(lhs.pow(&rhs)));
                    }
                    _ => return Err(self.error(InternalRuntimeError::TypeError)),
                }
            }
            Instruction::Negate => {
                let val = self.stack_pop()?;
                match -val {
                    Ok(val) => self.stack.push(val),
                    Err(..) => return Err(self.error(InternalRuntimeError::TypeError)),
                }
            }
            Instruction::Glue => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match (lhs, rhs) {
                    (Value::String(lhs), Value::String(rhs)) => {
                        self.stack.push(Value::String(lhs + &rhs))
                    }
                    (Value::Array(lhs), Value::Array(rhs)) => {
                        lhs.append(&rhs);
                        self.stack.push(Value::Array(lhs));
                    }
                    (Value::Set(lhs), Value::Set(rhs)) => {
                        lhs.union(&rhs);
                        self.stack.push(Value::Set(lhs));
                    }
                    (Value::Record(lhs), Value::Record(rhs)) => {
                        lhs.union(&rhs);
                        self.stack.push(Value::Record(lhs));
                    }
                    _ => return Err(self.error(InternalRuntimeError::TypeError)),
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
                        .ok_or(self.error(InternalRuntimeError::TypeError))?,
                    _ => return Err(self.error(InternalRuntimeError::TypeError)),
                };
                match lhs {
                    Value::String(lhs) => self
                        .stack
                        .push(Value::String(lhs.chars().skip(count).collect())),
                    Value::Array(lhs) => {
                        self.stack.push(Value::Array(lhs.range(count..).to_owned()))
                    }
                    _ => return Err(self.error(InternalRuntimeError::TypeError)),
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
                        .ok_or(self.error(InternalRuntimeError::TypeError))?,
                    _ => return Err(self.error(InternalRuntimeError::TypeError)),
                };
                match lhs {
                    Value::String(lhs) => {
                        self.stack
                            .push(Value::String(lhs.chars().take(count).collect()));
                    }
                    Value::Array(lhs) => {
                        self.stack.push(Value::Array(lhs.range(..count).to_owned()));
                    }
                    _ => return Err(self.error(InternalRuntimeError::TypeError)),
                }
            }
            Instruction::Access => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match (lhs, rhs) {
                    (Value::Record(record), rhs) => match record.get(&rhs) {
                        Some(value) => self.stack.push(value),
                        None => return Err(self.error(InternalRuntimeError::TypeError)),
                    },
                    (Value::String(lhs), Value::Number(rhs)) => {
                        let ch = rhs
                            .as_uinteger()
                            .and_then(|index| index.to_usize())
                            .and_then(|index| lhs.chars().nth(index));
                        match ch {
                            Some(ch) => self.stack.push(Value::Char(ch)),
                            None => return Err(self.error(InternalRuntimeError::TypeError)),
                        }
                    }
                    (Value::Bits(lhs), Value::Number(rhs)) => {
                        let val = rhs
                            .as_uinteger()
                            .and_then(|index| index.to_usize())
                            .and_then(|index| lhs.get(index));
                        match val {
                            Some(val) => self.stack.push(Value::Bool(val)),
                            None => return Err(self.error(InternalRuntimeError::TypeError)),
                        }
                    }
                    (Value::Array(lhs), Value::Number(rhs)) => {
                        let val = rhs
                            .as_uinteger()
                            .and_then(|index| index.to_usize())
                            .and_then(|index| lhs.get(index));
                        match val {
                            Some(val) => self.stack.push(val),
                            None => return Err(self.error(InternalRuntimeError::TypeError)),
                        }
                    }
                    _ => return Err(self.error(InternalRuntimeError::TypeError)),
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
                            None => return Err(self.error(InternalRuntimeError::TypeError)),
                        }
                    }
                    _ => return Err(self.error(InternalRuntimeError::TypeError)),
                }
                self.stack.push(lhs);
            }
            Instruction::Length => {
                let value = self.stack_pop()?;
                match value {
                    Value::Array(arr) => self.stack.push(Value::from(arr.len())),
                    Value::Record(record) => self.stack.push(Value::from(record.len())),
                    Value::Set(set) => self.stack.push(Value::from(set.len())),
                    Value::String(string) => self.stack.push(Value::from(string.len())),
                    Value::Bits(bits) => self.stack.push(Value::from(bits.len())),
                    _ => return Err(self.error(InternalRuntimeError::TypeError)),
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
                    _ => return Err(self.error(InternalRuntimeError::TypeError)),
                }
                self.stack.push(collection);
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
                            return Err(self.error(InternalRuntimeError::TypeError));
                        };
                        let Some(index) = number.as_uinteger().and_then(|index| index.to_usize())
                        else {
                            return Err(self.error(InternalRuntimeError::TypeError));
                        };
                        arr.remove(index);
                    }
                    _ => return Err(self.error(InternalRuntimeError::TypeError)),
                }
                self.stack.push(value);
            }
            Instruction::Contains => {
                let key = self.stack_pop()?;
                let value = self.stack_pop()?;
                match value {
                    Value::Record(record) => {
                        self.stack.push(record.contains_key(&key));
                    }
                    Value::Set(set) => {
                        self.stack.push(set.has(&key));
                    }
                    Value::Array(arr) => {
                        self.stack.push(arr.contains(&key));
                    }
                    _ => return Err(self.error(InternalRuntimeError::TypeError)),
                }
            }
            Instruction::Entries => {
                let collection = self.stack_pop()?;
                match collection {
                    Value::Record(record) => {
                        self.stack
                            .push(record.into_iter().map(Into::into).collect::<Vec<Value>>());
                    }
                    Value::Set(set) => {
                        self.stack.push(set.into_iter().collect::<Vec<Value>>());
                    }
                    value @ Value::Array(..) => {
                        self.stack.push(value);
                    }
                    _ => return Err(self.error(InternalRuntimeError::TypeError)),
                }
            }
            Instruction::Not => {
                let val = self.stack_pop()?;
                match val {
                    Value::Bool(val) => self.stack.push(Value::Bool(!val)),
                    _ => return Err(self.error(InternalRuntimeError::TypeError)),
                }
            }
            Instruction::And => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match (lhs, rhs) {
                    (Value::Bool(lhs), Value::Bool(rhs)) => {
                        self.stack.push(Value::Bool(lhs && rhs))
                    }
                    _ => return Err(self.error(InternalRuntimeError::TypeError)),
                }
            }
            Instruction::Or => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match (lhs, rhs) {
                    (Value::Bool(lhs), Value::Bool(rhs)) => {
                        self.stack.push(Value::Bool(lhs || rhs))
                    }
                    _ => return Err(self.error(InternalRuntimeError::TypeError)),
                }
            }
            Instruction::BitwiseAnd => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match lhs & rhs {
                    Ok(val) => self.stack.push(val),
                    _ => return Err(self.error(InternalRuntimeError::TypeError)),
                }
            }
            Instruction::BitwiseOr => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match lhs | rhs {
                    Ok(val) => self.stack.push(val),
                    _ => return Err(self.error(InternalRuntimeError::TypeError)),
                }
            }
            Instruction::BitwiseXor => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match lhs ^ rhs {
                    Ok(val) => self.stack.push(val),
                    _ => return Err(self.error(InternalRuntimeError::TypeError)),
                }
            }
            Instruction::BitwiseNeg => {
                let val = self.stack_pop()?;
                match val {
                    Value::Bits(val) => self.stack.push(Value::Bits(!val)),
                    _ => return Err(self.error(InternalRuntimeError::TypeError)),
                }
            }
            Instruction::LeftShift => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match lhs << rhs {
                    Ok(val) => self.stack.push(val),
                    _ => return Err(self.error(InternalRuntimeError::TypeError)),
                }
            }
            Instruction::RightShift => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                match lhs >> rhs {
                    Ok(val) => self.stack.push(val),
                    _ => return Err(self.error(InternalRuntimeError::TypeError)),
                }
            }
            Instruction::Cons => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                self.stack.push(Value::Tuple(Tuple::new(lhs, rhs)));
            }
            Instruction::Uncons => {
                let (first, second) = self.stack_pop().and_then(|val| match val {
                    Value::Tuple(tuple) => Ok(tuple.uncons()),
                    _ => Err(self.error(InternalRuntimeError::TypeError)),
                })?;
                self.stack.push(first);
                self.stack.push(second);
            }
            Instruction::First => {
                let first = self.stack_pop().and_then(|val| match val {
                    Value::Tuple(tuple) => Ok(tuple.into_first()),
                    _ => Err(self.error(InternalRuntimeError::TypeError)),
                })?;
                self.stack.push(first);
            }
            Instruction::Second => {
                let first = self.stack_pop().and_then(|val| match val {
                    Value::Tuple(tuple) => Ok(tuple.into_second()),
                    _ => Err(self.error(InternalRuntimeError::TypeError)),
                })?;
                self.stack.push(first);
            }
            Instruction::Construct => {
                let atom = self.stack_pop().and_then(|val| match val {
                    Value::Atom(atom) => Ok(atom),
                    _ => Err(self.error(InternalRuntimeError::TypeError)),
                })?;
                let value = self.stack_pop()?;
                self.stack.push(Value::Struct(Struct::new(atom, value)));
            }
            Instruction::Destruct => {
                let (atom, value) = self.stack_pop().and_then(|val| match val {
                    Value::Struct(val) => Ok(val.destruct()),
                    _ => Err(self.error(InternalRuntimeError::TypeError)),
                })?;
                self.stack.push(value);
                self.stack.push(atom);
            }
            Instruction::Leq => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                let cmp = match lhs.partial_cmp(&rhs) {
                    Some(Ordering::Less | Ordering::Equal) => Value::Bool(true),
                    Some(Ordering::Greater) => Value::Bool(false),
                    None => Value::Unit,
                };
                self.stack.push(cmp);
            }
            Instruction::Lt => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                let cmp = match lhs.partial_cmp(&rhs) {
                    Some(Ordering::Less) => Value::Bool(true),
                    Some(Ordering::Greater) | Some(Ordering::Equal) => Value::Bool(false),
                    None => Value::Unit,
                };
                self.stack.push(cmp);
            }
            Instruction::Geq => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                let cmp = match lhs.partial_cmp(&rhs) {
                    Some(Ordering::Less) => Value::Bool(false),
                    Some(Ordering::Greater) | Some(Ordering::Equal) => Value::Bool(true),
                    None => Value::Unit,
                };
                self.stack.push(cmp);
            }
            Instruction::Gt => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                let cmp = match lhs.partial_cmp(&rhs) {
                    Some(Ordering::Less) | Some(Ordering::Equal) => Value::Bool(false),
                    Some(Ordering::Greater) => Value::Bool(true),
                    None => Value::Unit,
                };
                self.stack.push(cmp);
            }
            Instruction::RefEq => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                self.stack.push(Value::Bool(ReferentialEq::eq(&lhs, &rhs)));
            }
            Instruction::ValEq => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                self.stack.push(Value::Bool(StructuralEq::eq(&lhs, &rhs)));
            }
            Instruction::RefNeq => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                self.stack.push(Value::Bool(!ReferentialEq::eq(&lhs, &rhs)));
            }
            Instruction::ValNeq => {
                let rhs = self.stack_pop()?;
                let lhs = self.stack_pop()?;
                self.stack.push(Value::Bool(!StructuralEq::eq(&lhs, &rhs)));
            }
            Instruction::Call(arity) => {
                let arguments = self
                    .stack
                    .pop_n(arity as usize)
                    .map_err(|k| self.error(k))?;
                let callable = self.stack_pop()?;
                self.call_internal(callable, arguments)?;
            }
            Instruction::Become(arity) => {
                self.r#become(arity as usize)?;
            }
            Instruction::Return => {
                let return_value = self.stack_pop()?;
                self.r#return(return_value)?;
            }
            Instruction::Close(offset) => {
                let closure = Closure::new(self.ip, self.stack.branch());
                self.stack.push(Value::from(closure));
                self.ip = offset;
            }
            Instruction::Shift(offset) => {
                let continuation = Continuation::new(self.ip, self.stack.branch());
                self.stack.push(Value::from(continuation));
                self.ip = offset;
            }
            Instruction::Jump(offset) => {
                self.ip = offset;
            }
            Instruction::CondJump(offset) => {
                let cond = self.stack_pop()?;
                match cond {
                    Value::Bool(false) => self.ip = offset,
                    Value::Bool(true) => {}
                    _ => return Err(self.error(InternalRuntimeError::TypeError)),
                }
            }
            Instruction::Branch => {
                // A branch requires two values on the stack; the two branches get the
                // different values, respectively.
                let right = self.stack_pop()?;
                let left = self.stack_pop()?;
                let mut branch = self.branch();
                self.stack.push(left);
                branch.stack.push(right);
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
            Instruction::Panic => {
                let value = self.stack_pop()?;
                return Err(self.error(ErrorKind::RuntimeError(value)));
            }
            Instruction::Chunk(locator) => {
                let value = self.program.locate(locator).map_err(|er| self.error(er))?;
                self.stack.push(value);
            }
            Instruction::Debug => {
                let val = self.stack.at_raw(0).map_err(|k| self.error(k))?;
                eprintln!("{}", val);
            }
        }
        Ok(Step::Continue)
    }
}
