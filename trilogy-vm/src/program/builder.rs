use crate::bytecode::{LabelAlreadyInserted, Offset, OpCode};
use crate::runtime::atom::AtomInterner;
use crate::runtime::Procedure;
use crate::traits::Tags;
use crate::{Atom, Instruction, Program, Value};
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug)]
pub struct UnknownLabel(String);
impl std::error::Error for UnknownLabel {}
impl Display for UnknownLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown label \"{}\"", self.0)
    }
}

#[derive(Default)]
pub struct ProgramBuilder {
    interner: AtomInterner,
    labels: HashMap<String, Offset>,
    constants: Vec<Value>,
    constant_holes: HashMap<usize, String>,
    bytes: Vec<u8>,
    byte_holes: HashMap<usize, String>,
}

impl ProgramBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write_opcode(&mut self, opcode: OpCode) -> &mut Self {
        self.bytes.push(opcode as u8);
        self
    }

    pub fn write_offset(&mut self, offset: usize) -> &mut Self {
        self.bytes.extend((offset as u32).to_be_bytes());
        self
    }

    pub fn write_offset_label(&mut self, label: String) -> &mut Self {
        self.byte_holes.insert(self.bytes.len(), label);
        self.bytes.extend(0u32.to_be_bytes());
        self
    }

    pub fn write_reuse_constant(&mut self, offset: usize) -> &mut Self {
        self.write_opcode(OpCode::Const);
        self.write_offset(offset);
        self
    }

    /// Writes the next instruction.
    pub fn write_instruction(&mut self, instruction: Instruction) -> &mut Self {
        self.write_opcode(instruction.tag());
        let offset = match instruction {
            Instruction::Const(constant) => {
                let index = self.write_constant(constant);
                Some(index)
            }
            Instruction::LoadLocal(offset) => Some(offset),
            Instruction::SetLocal(offset) => Some(offset),
            Instruction::LoadRegister(offset) => Some(offset),
            Instruction::SetRegister(offset) => Some(offset),
            Instruction::Call(offset) => Some(offset),
            Instruction::Shift(offset) => Some(offset),
            Instruction::Jump(offset) => Some(offset),
            Instruction::JumpBack(offset) => Some(offset),
            Instruction::CondJump(offset) => Some(offset),
            Instruction::CondJumpBack(offset) => Some(offset),
            _ => None,
        };
        if let Some(offset) = offset {
            self.write_offset(offset);
        }
        self
    }

    /// Writes a label at the position of the next instruction in the program. Returns the
    /// offset of that label, or an error if a label with this name has already been set.
    pub fn write_label(&mut self, label: String) -> Result<&mut Self, LabelAlreadyInserted> {
        if self.labels.insert(label, self.bytes.len()).is_some() {
            return Err(LabelAlreadyInserted);
        }
        Ok(self)
    }

    /// Stores a label as a value in the constant table. The value will be resolved later.
    pub fn store_label(&mut self, label: String) -> usize {
        let index = self.constants.len();
        self.constants.push(Value::Unit);
        self.constant_holes.insert(index, label);
        index
    }

    /// Stores a value in the constants table.
    pub fn write_constant(&mut self, value: Value) -> usize {
        let index = self.constants.len();
        self.constants.push(value);
        index
    }

    /// Creates an Atom from a string. Atoms are unique within the scope of a program, so
    /// cannot be created externally.
    pub fn atom(&mut self, value: &str) -> Atom {
        self.interner.intern(value)
    }

    /// Retrieves the offset of the next instruction to be written.
    pub fn ip(&self) -> Offset {
        self.bytes.len()
    }

    /// Completes the building of the program, returning the complete program or an
    /// error indicating why the program is invalid.
    pub fn build(mut self) -> Result<Program, UnknownLabel> {
        for (constant, label) in self.constant_holes.into_iter() {
            let offset = self.labels.get(&label).ok_or(UnknownLabel(label))?;
            self.constants[constant] = Value::Procedure(Procedure::new(*offset));
        }
        for (ip, label) in self.byte_holes.into_iter() {
            let offset = *self.labels.get(&label).ok_or(UnknownLabel(label))?;
            if ip < offset {
                // Jumping forwards
                let distance = offset - (ip + 4);
                self.bytes
                    .splice(ip..ip + 4, u32::to_be_bytes(distance as u32));
            } else {
                // Jumping backwards
                let distance = (ip + 4) - offset;
                self.bytes
                    .splice(ip..ip + 4, u32::to_be_bytes(distance as u32));
                self.bytes[ip - 1] += 1;
            }
        }
        Ok(Program {
            constants: self.constants,
            instructions: self.bytes,
            labels: self.labels,
        })
    }
}
