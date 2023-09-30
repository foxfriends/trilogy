use super::error::{ErrorKind, InternalRuntimeError};
use super::{Error, Execution};
use crate::atom::AtomInterner;
use crate::bytecode::{ChunkError, OpCode};
use crate::runtime::Number;
use crate::{
    Atom, Chunk, ChunkBuilder, Instruction, Procedure, Program, ReferentialEq, Struct, StructuralEq,
};
use crate::{Tuple, Value};
use num::ToPrimitive;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Clone, Debug)]
enum HeapCell {
    Unset,
    Value(Value),
    Freed,
}

impl HeapCell {
    fn unset(&mut self) -> Result<(), InternalRuntimeError> {
        if matches!(self, HeapCell::Freed) {
            Err(InternalRuntimeError::UseAfterFree)
        } else {
            *self = HeapCell::Unset;
            Ok(())
        }
    }

    fn set(&mut self, value: Value) -> Result<(), InternalRuntimeError> {
        if matches!(self, HeapCell::Freed) {
            Err(InternalRuntimeError::UseAfterFree)
        } else {
            *self = HeapCell::Value(value);
            Ok(())
        }
    }

    fn init(&mut self, value: Value) -> Result<bool, InternalRuntimeError> {
        match self {
            HeapCell::Unset => {
                *self = HeapCell::Value(value);
                Ok(true)
            }
            HeapCell::Value(..) => Ok(false),
            HeapCell::Freed => Err(InternalRuntimeError::UseAfterFree),
        }
    }

    fn as_value(&mut self) -> Result<Option<Value>, InternalRuntimeError> {
        match self {
            HeapCell::Unset => Ok(None),
            HeapCell::Value(val) => Ok(Some(val.clone())),
            HeapCell::Freed => Err(InternalRuntimeError::UseAfterFree),
        }
    }
}

/// The actual Trilogy Virtual Machine.
///
/// This is a stack-based VM, but also with registers and heap.
/// Further documentation on the actual specifics will follow.
#[derive(Clone, Debug)]
pub struct VirtualMachine {
    atom_interner: AtomInterner,
    executions: VecDeque<Execution>,
    registers: Vec<Value>,
    heap: Vec<HeapCell>,
}

impl Default for VirtualMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualMachine {
    /// Creates a new virtual machine with empty heap and registers.
    pub fn new() -> Self {
        Self {
            atom_interner: AtomInterner::default(),
            executions: VecDeque::with_capacity(8),
            registers: vec![Value::Unit; 8],
            heap: Vec::with_capacity(8),
        }
    }

    /// Create an atom in the context of this VM.
    pub fn atom(&self, atom: &str) -> Atom {
        self.atom_interner.intern(atom)
    }

    /// Compiles a program to bytecode, returning the compiled [`Chunk`][].
    pub fn compile(&self, program: &dyn Program) -> Result<Chunk, ChunkError> {
        let mut chunks_compiled = HashSet::new();
        let mut dependencies = vec![];

        let mut chunk_builder = ChunkBuilder::new(self.atom_interner.clone());
        program.entrypoint(&mut chunk_builder);
        let (_, mut chunk) = chunk_builder.build()?;
        loop {
            dependencies.extend(
                chunk
                    .iter()
                    .filter_map(|instruction| match instruction {
                        Instruction::Chunk(val) => Some(val),
                        _ => None,
                    })
                    .filter(|val| !chunks_compiled.contains(val)),
            );
            let Some(dep) = dependencies.pop() else {
                return Ok(chunk);
            };
            let mut chunk_builder = ChunkBuilder::new_prefixed(self.atom_interner.clone(), chunk);
            program.chunk(&dep, &mut chunk_builder);
            chunks_compiled.insert(dep);
            (_, chunk) = chunk_builder.build()?;
        }
    }

    /// Run a [`Program`][] on this VM.
    ///
    /// This mutates the VM's internal state (heap and registers), so if reusing `VirtualMachine`
    /// instances, be sure this is the expected behaviour.
    pub fn run(&mut self, program: &dyn Program) -> Result<Value, Error> {
        let mut chunk_builder = ChunkBuilder::new(self.atom_interner.clone());
        let mut chunk_cache = HashMap::<Value, Value>::new();
        program.entrypoint(&mut chunk_builder);
        let mut ex = Execution::new();
        let (entrypoint, mut chunk) = chunk_builder
            .build()
            .map_err(|err| ex.error(ErrorKind::InvalidBytecode(err)))?;
        ex.ip = entrypoint;
        self.executions.push_back(ex);
        // In future, multiple executions will likely be run in parallel on different
        // threads. Maybe this should be a compile time option, where the alternatives
        // are depth-first, or breadth-first multitasking.
        //
        // For now, the only option is depth-first multitasking, as it is easiest to
        // reason about and implement, as well as most likely to be performant in most
        // simple situations, which is all that we will have to deal with while this
        // VM remains a (relative) toy.
        let ep = 0;
        let last_ex = loop {
            let ex = &mut self.executions[ep];
            let instruction = ex.read_opcode(&chunk)?;
            match instruction {
                OpCode::Const => {
                    let constant = ex.read_constant(&chunk)?;
                    ex.stack_push(constant);
                }
                OpCode::Load => {
                    let pointer = ex.stack_pop_pointer()?;
                    ex.stack_push(
                        self.heap
                            .get(pointer)
                            .cloned()
                            .ok_or_else(|| ex.error(InternalRuntimeError::InvalidPointer))?
                            .as_value()
                            .map_err(|er| ex.error(er))?
                            .ok_or_else(|| ex.error(InternalRuntimeError::ExpectedValue))?,
                    );
                }
                OpCode::Set => {
                    let pointer = ex.stack_pop_pointer()?;
                    let value = ex.stack_pop()?;
                    self.heap
                        .get_mut(pointer)
                        .ok_or_else(|| ex.error(InternalRuntimeError::InvalidPointer))?
                        .set(value)
                        .map_err(|er| ex.error(er))?;
                }
                OpCode::Init => {
                    let pointer = ex.stack_pop_pointer()?;
                    let value = ex.stack_pop()?;
                    let was_set = self
                        .heap
                        .get_mut(pointer)
                        .ok_or_else(|| ex.error(InternalRuntimeError::InvalidPointer))?
                        .init(value)
                        .map_err(|er| ex.error(er))?;
                    ex.stack_push(was_set.into());
                }
                OpCode::Unset => {
                    let pointer = ex.stack_pop_pointer()?;
                    self.heap
                        .get_mut(pointer)
                        .ok_or_else(|| ex.error(InternalRuntimeError::InvalidPointer))?
                        .unset()
                        .map_err(|er| ex.error(er))?;
                }
                OpCode::Alloc => {
                    let pointer = self.heap.len();
                    self.heap.push(HeapCell::Unset);
                    ex.stack_push_pointer(pointer);
                }
                OpCode::Free => {
                    let pointer = ex.stack_pop_pointer()?;
                    self.heap[pointer] = HeapCell::Freed;
                }
                OpCode::LoadLocal => {
                    let offset = ex.read_offset(&chunk)? as usize;
                    ex.stack_push(ex.read_local(offset)?);
                }
                OpCode::Variable => {
                    ex.push_unset();
                }
                OpCode::SetLocal => {
                    let offset = ex.read_offset(&chunk)? as usize;
                    let value = ex.stack_pop()?;
                    ex.set_local(offset, value)?;
                }
                OpCode::UnsetLocal => {
                    let offset = ex.read_offset(&chunk)? as usize;
                    ex.unset_local(offset)?;
                }
                OpCode::InitLocal => {
                    let offset = ex.read_offset(&chunk)? as usize;
                    let value = ex.stack_pop()?;
                    let did_set = ex.init_local(offset, value)?;
                    ex.stack_push(did_set.into());
                }
                OpCode::LoadRegister => {
                    let offset = ex.read_offset(&chunk)? as usize;
                    if offset >= self.registers.len() {
                        return Err(ex.error(InternalRuntimeError::InvalidRegister));
                    }
                    ex.stack_push(self.registers[offset].clone());
                }
                OpCode::SetRegister => {
                    let offset = ex.read_offset(&chunk)? as usize;
                    let value = ex.stack_pop()?;
                    if offset >= self.registers.len() {
                        return Err(ex.error(InternalRuntimeError::InvalidRegister));
                    }
                    self.registers[offset] = value;
                }
                OpCode::Pop => {
                    ex.stack_discard()?;
                }
                OpCode::Swap => {
                    ex.stack_slide(1)?;
                }
                OpCode::Slide => {
                    let offset = ex.read_offset(&chunk)? as usize;
                    ex.stack_slide(offset)?;
                }
                OpCode::Copy => {
                    let value = ex.stack_peek()?;
                    ex.stack_push(value);
                }
                OpCode::Clone => {
                    let value = ex.stack_pop()?;
                    ex.stack_push(value.structural_clone());
                }
                OpCode::TypeOf => {
                    let value = ex.stack_pop()?;
                    match value {
                        Value::Unit => ex.stack_push("unit".into()),
                        Value::Number(..) => ex.stack_push("number".into()),
                        Value::Bits(..) => ex.stack_push("bits".into()),
                        Value::Bool(..) => ex.stack_push("boolean".into()),
                        Value::String(..) => ex.stack_push("string".into()),
                        Value::Char(..) => ex.stack_push("character".into()),
                        Value::Tuple(..) => ex.stack_push("tuple".into()),
                        Value::Array(..) => ex.stack_push("array".into()),
                        Value::Set(..) => ex.stack_push("set".into()),
                        Value::Record(..) => ex.stack_push("record".into()),
                        Value::Atom(..) => ex.stack_push("atom".into()),
                        Value::Struct(..) => ex.stack_push("struct".into()),
                        Value::Procedure(..) | Value::Continuation(..) | Value::Native(..) => {
                            ex.stack_push("callable".into())
                        }
                    }
                }
                OpCode::Add => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    match lhs + rhs {
                        Ok(val) => ex.stack_push(val),
                        Err(..) => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                }
                OpCode::Subtract => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    match lhs - rhs {
                        Ok(val) => ex.stack_push(val),
                        Err(..) => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                }
                OpCode::Multiply => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    match lhs * rhs {
                        Ok(val) => ex.stack_push(val),
                        Err(..) => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                }
                OpCode::Divide => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    match lhs / rhs {
                        Ok(val) => ex.stack_push(val),
                        Err(..) => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                }
                OpCode::Remainder => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    match lhs % rhs {
                        Ok(val) => ex.stack_push(val),
                        Err(..) => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                }
                OpCode::IntDivide => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    match lhs / rhs {
                        Ok(Value::Number(val)) => {
                            ex.stack_push(Value::Number(Number::from(val.as_complex().re.floor())));
                        }
                        _ => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                }
                OpCode::Power => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    match (lhs, rhs) {
                        (Value::Number(lhs), Value::Number(rhs)) => {
                            ex.stack_push(Value::Number(lhs.pow(&rhs)));
                        }
                        _ => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                }
                OpCode::Negate => {
                    let val = ex.stack_pop()?;
                    match -val {
                        Ok(val) => ex.stack_push(val),
                        Err(..) => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                }
                OpCode::Glue => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    match (lhs, rhs) {
                        (Value::String(lhs), Value::String(rhs)) => {
                            ex.stack_push(Value::String(lhs + &rhs))
                        }
                        (Value::Array(lhs), Value::Array(rhs)) => {
                            lhs.append(&rhs);
                            ex.stack_push(Value::Array(lhs));
                        }
                        (Value::Set(lhs), Value::Set(rhs)) => {
                            lhs.union(&rhs);
                            ex.stack_push(Value::Set(lhs));
                        }
                        (Value::Record(lhs), Value::Record(rhs)) => {
                            lhs.union(&rhs);
                            ex.stack_push(Value::Record(lhs));
                        }
                        _ => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                }
                OpCode::Skip => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    let count = match rhs {
                        Value::Number(number) if number.is_uinteger() => number
                            .as_uinteger()
                            .unwrap()
                            .to_usize()
                            .ok_or(ex.error(ErrorKind::RuntimeTypeError))?,
                        _ => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    };
                    match lhs {
                        Value::String(lhs) => {
                            ex.stack_push(Value::String(lhs.chars().skip(count).collect()))
                        }
                        Value::Array(lhs) => {
                            ex.stack_push(Value::Array(lhs.range(count..).to_owned()))
                        }
                        _ => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                }
                OpCode::Take => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    let count = match rhs {
                        Value::Number(number) if number.is_uinteger() => number
                            .as_uinteger()
                            .unwrap()
                            .to_usize()
                            .ok_or(ex.error(ErrorKind::RuntimeTypeError))?,
                        _ => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    };
                    match lhs {
                        Value::String(lhs) => {
                            ex.stack_push(Value::String(lhs.chars().take(count).collect()));
                        }
                        Value::Array(lhs) => {
                            ex.stack_push(Value::Array(lhs.range(..count).to_owned()));
                        }
                        _ => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                }
                OpCode::Access => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    match (lhs, rhs) {
                        (Value::Record(record), rhs) => match record.get(&rhs) {
                            Some(value) => ex.stack_push(value),
                            None => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                        },
                        (Value::String(lhs), Value::Number(rhs)) => {
                            let ch = rhs
                                .as_uinteger()
                                .and_then(|index| index.to_usize())
                                .and_then(|index| lhs.chars().nth(index));
                            match ch {
                                Some(ch) => ex.stack_push(Value::Char(ch)),
                                None => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                            }
                        }
                        (Value::Bits(lhs), Value::Number(rhs)) => {
                            let val = rhs
                                .as_uinteger()
                                .and_then(|index| index.to_usize())
                                .and_then(|index| lhs.get(index));
                            match val {
                                Some(val) => ex.stack_push(Value::Bool(val)),
                                None => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                            }
                        }
                        (Value::Array(lhs), Value::Number(rhs)) => {
                            let val = rhs
                                .as_uinteger()
                                .and_then(|index| index.to_usize())
                                .and_then(|index| lhs.get(index));
                            match val {
                                Some(val) => ex.stack_push(val),
                                None => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                            }
                        }
                        _ => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                }
                OpCode::Assign => {
                    let value = ex.stack_pop()?;
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    match (&lhs, rhs, value) {
                        (Value::Record(record), rhs, value) => {
                            record.insert(rhs, value);
                        }
                        (Value::Array(lhs), Value::Number(rhs), value) => {
                            let index = rhs.as_uinteger().and_then(|index| index.to_usize());
                            match index {
                                Some(index) => lhs.set(index, value),
                                None => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                            }
                        }
                        _ => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                    ex.stack_push(lhs);
                }
                OpCode::Length => {
                    let value = ex.stack_pop()?;
                    match value {
                        Value::Array(arr) => ex.stack_push(Value::from(arr.len())),
                        Value::Record(record) => ex.stack_push(Value::from(record.len())),
                        Value::Set(set) => ex.stack_push(Value::from(set.len())),
                        Value::String(string) => ex.stack_push(Value::from(string.len())),
                        _ => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                }
                OpCode::Insert => {
                    let value = ex.stack_pop()?;
                    let collection = ex.stack_pop()?;
                    match &collection {
                        Value::Array(arr) => {
                            arr.push(value);
                        }
                        Value::Set(set) => {
                            set.insert(value);
                        }
                        _ => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                    ex.stack_push(collection);
                }
                OpCode::Delete => {
                    let key = ex.stack_pop()?;
                    let value = ex.stack_pop()?;
                    match &value {
                        Value::Record(record) => {
                            record.remove(&key);
                        }
                        Value::Set(set) => {
                            set.remove(&key);
                        }
                        Value::Array(arr) => {
                            let Value::Number(number) = key else {
                                return Err(ex.error(ErrorKind::RuntimeTypeError));
                            };
                            let Some(index) =
                                number.as_uinteger().and_then(|index| index.to_usize())
                            else {
                                return Err(ex.error(ErrorKind::RuntimeTypeError));
                            };
                            arr.remove(index);
                        }
                        _ => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                    ex.stack_push(value);
                }
                OpCode::Contains => {
                    let key = ex.stack_pop()?;
                    let value = ex.stack_pop()?;
                    match value {
                        Value::Record(record) => {
                            ex.stack_push(record.contains_key(&key).into());
                        }
                        Value::Set(set) => {
                            ex.stack_push(set.has(&key).into());
                        }
                        Value::Array(arr) => {
                            ex.stack_push(arr.contains(&key).into());
                        }
                        _ => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                }
                OpCode::Entries => {
                    let collection = ex.stack_pop()?;
                    match collection {
                        Value::Record(record) => {
                            ex.stack_push(
                                record
                                    .into_iter()
                                    .map(Into::into)
                                    .collect::<Vec<Value>>()
                                    .into(),
                            );
                        }
                        Value::Set(set) => {
                            ex.stack_push(set.into_iter().collect::<Vec<Value>>().into());
                        }
                        value @ Value::Array(..) => {
                            ex.stack_push(value);
                        }
                        _ => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                }
                OpCode::Not => {
                    let val = ex.stack_pop()?;
                    match val {
                        Value::Bool(val) => ex.stack_push(Value::Bool(!val)),
                        _ => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                }
                OpCode::And => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    match (lhs, rhs) {
                        (Value::Bool(lhs), Value::Bool(rhs)) => {
                            ex.stack_push(Value::Bool(lhs && rhs))
                        }
                        _ => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                }
                OpCode::Or => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    match (lhs, rhs) {
                        (Value::Bool(lhs), Value::Bool(rhs)) => {
                            ex.stack_push(Value::Bool(lhs || rhs))
                        }
                        _ => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                }
                OpCode::BitwiseAnd => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    match lhs & rhs {
                        Ok(val) => ex.stack_push(val),
                        _ => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                }
                OpCode::BitwiseOr => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    match lhs | rhs {
                        Ok(val) => ex.stack_push(val),
                        _ => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                }
                OpCode::BitwiseXor => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    match lhs ^ rhs {
                        Ok(val) => ex.stack_push(val),
                        _ => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                }
                OpCode::BitwiseNeg => {
                    let val = ex.stack_pop()?;
                    match val {
                        Value::Bits(val) => ex.stack_push(Value::Bits(!val)),
                        _ => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                }
                OpCode::LeftShift => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    match lhs << rhs {
                        Ok(val) => ex.stack_push(val),
                        _ => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                }
                OpCode::RightShift => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    match lhs >> rhs {
                        Ok(val) => ex.stack_push(val),
                        _ => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                }
                OpCode::Cons => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    ex.stack_push(Value::Tuple(Tuple::new(lhs, rhs)));
                }
                OpCode::Uncons => {
                    let (first, second) = ex.stack_pop().and_then(|val| match val {
                        Value::Tuple(tuple) => Ok(tuple.uncons()),
                        _ => Err(ex.error(ErrorKind::RuntimeTypeError)),
                    })?;
                    ex.stack_push(first);
                    ex.stack_push(second);
                }
                OpCode::First => {
                    let first = ex.stack_pop().and_then(|val| match val {
                        Value::Tuple(tuple) => Ok(tuple.into_first()),
                        _ => Err(ex.error(ErrorKind::RuntimeTypeError)),
                    })?;
                    ex.stack_push(first);
                }
                OpCode::Second => {
                    let first = ex.stack_pop().and_then(|val| match val {
                        Value::Tuple(tuple) => Ok(tuple.into_second()),
                        _ => Err(ex.error(ErrorKind::RuntimeTypeError)),
                    })?;
                    ex.stack_push(first);
                }
                OpCode::Construct => {
                    let rhs = ex.stack_pop()?;
                    let atom = ex.stack_pop().and_then(|val| match val {
                        Value::Atom(atom) => Ok(atom),
                        _ => Err(ex.error(ErrorKind::RuntimeTypeError)),
                    })?;
                    ex.stack_push(Value::Struct(Struct::new(atom, rhs)));
                }
                OpCode::Destruct => {
                    let (atom, value) = ex.stack_pop().and_then(|val| match val {
                        Value::Struct(val) => Ok(val.destruct()),
                        _ => Err(ex.error(ErrorKind::RuntimeTypeError)),
                    })?;
                    ex.stack_push(atom.into());
                    ex.stack_push(value);
                }
                OpCode::Leq => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    let cmp = match lhs.partial_cmp(&rhs) {
                        Some(Ordering::Less | Ordering::Equal) => Value::Bool(true),
                        Some(Ordering::Greater) => Value::Bool(false),
                        None => Value::Unit,
                    };
                    ex.stack_push(cmp);
                }
                OpCode::Lt => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    let cmp = match lhs.partial_cmp(&rhs) {
                        Some(Ordering::Less) => Value::Bool(true),
                        Some(Ordering::Greater) | Some(Ordering::Equal) => Value::Bool(false),
                        None => Value::Unit,
                    };
                    ex.stack_push(cmp);
                }
                OpCode::Geq => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    let cmp = match lhs.partial_cmp(&rhs) {
                        Some(Ordering::Less) => Value::Bool(false),
                        Some(Ordering::Greater) | Some(Ordering::Equal) => Value::Bool(true),
                        None => Value::Unit,
                    };
                    ex.stack_push(cmp);
                }
                OpCode::Gt => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    let cmp = match lhs.partial_cmp(&rhs) {
                        Some(Ordering::Less) | Some(Ordering::Equal) => Value::Bool(false),
                        Some(Ordering::Greater) => Value::Bool(true),
                        None => Value::Unit,
                    };
                    ex.stack_push(cmp);
                }
                OpCode::RefEq => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    ex.stack_push(Value::Bool(ReferentialEq::eq(&lhs, &rhs)));
                }
                OpCode::ValEq => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    ex.stack_push(Value::Bool(StructuralEq::eq(&lhs, &rhs)));
                }
                OpCode::RefNeq => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    ex.stack_push(Value::Bool(!ReferentialEq::eq(&lhs, &rhs)));
                }
                OpCode::ValNeq => {
                    let rhs = ex.stack_pop()?;
                    let lhs = ex.stack_pop()?;
                    ex.stack_push(Value::Bool(!StructuralEq::eq(&lhs, &rhs)));
                }
                OpCode::Call => {
                    let arity = ex.read_offset(&chunk)?;
                    ex.call(arity as usize)?;
                }
                OpCode::Become => {
                    let arity = ex.read_offset(&chunk)?;
                    ex.r#become(arity as usize)?;
                }
                OpCode::Return => {
                    let return_value = ex.stack_pop()?;
                    ex.r#return()?;
                    ex.stack_push(return_value);
                }
                OpCode::Close => {
                    let offset = ex.read_offset(&chunk)?;
                    let closure = ex.current_closure();
                    ex.stack_push(Value::Procedure(closure));
                    ex.ip = offset;
                }
                OpCode::Shift => {
                    let offset = ex.read_offset(&chunk)?;
                    let continuation = ex.current_continuation();
                    ex.stack_push(Value::Continuation(continuation));
                    ex.ip = offset;
                }
                OpCode::Reset => {
                    let return_value = ex.stack_pop()?;
                    ex.reset_continuation()?;
                    ex.stack_push(return_value);
                }
                OpCode::Jump => {
                    let offset = ex.read_offset(&chunk)?;
                    ex.ip = offset;
                }
                OpCode::CondJump => {
                    let offset = ex.read_offset(&chunk)?;
                    let cond = ex.stack_pop()?;
                    match cond {
                        Value::Bool(false) => ex.ip = offset,
                        Value::Bool(true) => {}
                        _ => return Err(ex.error(ErrorKind::RuntimeTypeError)),
                    }
                }
                OpCode::Branch => {
                    // A branch requires two values on the stack; the two branches get the
                    // different values, respectively.
                    let right = ex.stack_pop()?;
                    let left = ex.stack_pop()?;
                    let mut branch = ex.branch();
                    ex.stack_push(left);
                    branch.stack_push(right);
                    self.executions.push_back(branch);
                }
                OpCode::Fizzle => {
                    // This just ends the execution.
                    //
                    // Is there any cleanup that has to be done? Or does Rust's
                    // RAII cause that cleanup to happen automatically?
                    //
                    // I suspect it is automatic.
                    let ex = self.executions.remove(ep).unwrap();
                    if self.executions.is_empty() {
                        break ex;
                    }
                }
                OpCode::Exit => {
                    // When run in embedded mode, the exit value can be any value. The
                    // interpreter binary can decide how to handle that exit value when
                    // passing off to the OS.
                    let value = ex.stack_pop()?;
                    self.executions.clear();
                    return Ok(value);
                }
                OpCode::Chunk => {
                    let locator = ex.read_constant(&chunk)?;
                    match chunk_cache.get(&locator) {
                        Some(cached) => ex.stack_push(cached.clone()),
                        None => {
                            let mut builder =
                                ChunkBuilder::new_prefixed(self.atom_interner.clone(), chunk);
                            program.chunk(&locator, &mut builder);
                            let entrypoint;
                            (entrypoint, chunk) = builder
                                .build()
                                .map_err(|err| ex.error(ErrorKind::InvalidBytecode(err)))?;
                            let chunk_procedure = Value::Procedure(Procedure::new(entrypoint));
                            chunk_cache.insert(locator, chunk_procedure.clone());
                            ex.stack_push(chunk_procedure);
                        }
                    }
                }
            }
        };
        Err(last_ex.error(ErrorKind::ExecutionFizzledError))
    }
}
