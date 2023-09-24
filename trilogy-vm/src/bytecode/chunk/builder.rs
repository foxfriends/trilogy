use super::Chunk;
use crate::atom::AtomInterner;
use crate::bytecode::asm::{self, AsmReader};
use crate::{Atom, Instruction, Offset, OpCode, Procedure, Value};
use std::fmt::{self, Display};

pub(super) enum Parameter {
    Value(Value),
    Label(String),
    Offset(u32),
    Reference(String),
}

struct Line {
    labels: Vec<String>,
    opcode: OpCode,
    value: Option<Parameter>,
}

/// Builder for constructing a chunk of bytecode for the [`VirtualMachine`][crate::VirtualMachine]
/// to execute.
pub struct ChunkBuilder {
    prefix: Option<Chunk>,
    entry_line: usize,
    interner: AtomInterner,
    lines: Vec<Line>,
    current_labels: Vec<String>,
    error: Option<ChunkError>,
}

/// An error that can occur when building a Chunk incorrectly.
#[derive(Clone, Debug)]
pub enum ChunkError {
    /// A referenced label was not defined.
    MissingLabel(String),
    /// Parsed assembly string was invalid.
    InvalidAsm,
}

impl Display for ChunkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingLabel(label) => write!(f, "label `{label}` was not defined"),
            Self::InvalidAsm => write!(f, "invalid assembly string was parsed"),
        }
    }
}

impl ChunkBuilder {
    pub(crate) fn new(interner: AtomInterner) -> Self {
        Self {
            entry_line: 0,
            prefix: None,
            interner,
            lines: vec![],
            current_labels: vec![],
            error: None,
        }
    }

    pub(crate) fn new_prefixed(interner: AtomInterner, prefix: Chunk) -> Self {
        Self {
            prefix: Some(prefix),
            ..Self::new(interner)
        }
    }

    /// Instantiate an atom for the current runtime. Atoms cannot be created except
    /// for within the context of a particular runtime's global atom table.
    pub fn atom(&self, atom: &str) -> Atom {
        self.interner.intern(atom)
    }

    /// Instantiate an anonymous atom for the current runtime. An anonymous atom
    /// cannot be re-created. The provided label is shown when debugging, but two
    /// atoms with the same label are not the same value.
    pub fn atom_anon(&self, label: &str) -> Atom {
        Atom::new_unique(label.to_owned())
    }

    /// Add a label to the next instruction to be inserted.
    ///
    /// ```asm
    /// label:
    /// ```
    ///
    /// Note that if no instruction is inserted following this label, the label will
    /// be treated as if it was not defined.
    pub fn label<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.current_labels.push(label.into());
        self
    }

    /// Insert a CONST instruction that references a procedure located at the
    /// given label.
    ///
    /// ```asm
    /// CONST &label
    /// ```
    pub fn reference<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.write_line(OpCode::Const, Some(Parameter::Reference(label.into())))
    }

    /// Insert a JUMP instruction to a given label.
    ///
    /// ```asm
    /// JUMP &label
    /// ```
    pub fn jump<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.write_line(OpCode::Jump, Some(Parameter::Label(label.into())))
    }

    /// Insert a JUMPF instruction to a given label.
    ///
    /// ```asm
    /// JUMPF &label
    /// ```
    pub fn cond_jump<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.write_line(OpCode::CondJump, Some(Parameter::Label(label.into())))
    }

    /// Insert a CLOSE instruction to a given label.
    ///
    /// ```asm
    /// CLOSE &label
    /// ```
    pub fn close<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.write_line(OpCode::Close, Some(Parameter::Label(label.into())))
    }

    /// Insert a SHIFT instruction to a given label.
    ///
    /// ```asm
    /// SHIFT &label
    /// ```
    pub fn shift<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.write_line(OpCode::Shift, Some(Parameter::Label(label.into())))
    }

    /// Insert an instruction.
    ///
    /// All labels currently in the buffer will be assigned to this line, and
    /// the buffer will be cleared.
    pub fn instruction(&mut self, instruction: Instruction) -> &mut Self {
        let opcode = instruction.op_code();
        let value = match instruction {
            Instruction::Const(value) => Some(Parameter::Value(value)),
            Instruction::LoadLocal(offset) => Some(Parameter::Offset(offset)),
            Instruction::SetLocal(offset) => Some(Parameter::Offset(offset)),
            Instruction::InitLocal(offset) => Some(Parameter::Offset(offset)),
            Instruction::UnsetLocal(offset) => Some(Parameter::Offset(offset)),
            Instruction::LoadRegister(offset) => Some(Parameter::Offset(offset)),
            Instruction::SetRegister(offset) => Some(Parameter::Offset(offset)),
            Instruction::Slide(offset) => Some(Parameter::Offset(offset)),
            Instruction::Call(offset) => Some(Parameter::Offset(offset)),
            Instruction::Become(offset) => Some(Parameter::Offset(offset)),
            Instruction::Close(offset) => Some(Parameter::Offset(offset)),
            Instruction::Shift(offset) => Some(Parameter::Offset(offset)),
            Instruction::Jump(offset) => Some(Parameter::Offset(offset)),
            Instruction::CondJump(offset) => Some(Parameter::Offset(offset)),
            _ => {
                assert_eq!(
                    instruction.byte_len(),
                    1,
                    "{instruction} needs to be handled"
                );
                None
            }
        };
        self.write_line(opcode, value)
    }

    pub(super) fn write_line(&mut self, opcode: OpCode, value: Option<Parameter>) -> &mut Self {
        let labels = self.current_labels.drain(..).collect();
        self.lines.push(Line {
            labels,
            opcode,
            value,
        });
        self
    }

    /// Sets the entrypoint of this chunk to be at the line about to be written.
    ///
    /// By default, the entrypoint is the start of the chunk, but this option may
    /// be used to start code execution from some point in the middle.
    pub fn entrypoint(&mut self) -> &mut Self {
        self.entry_line = self.lines.len();
        self
    }

    /// Construct the [`Chunk`][] that was being built. Fails if any labels were referenced
    /// but not defined.
    pub(crate) fn build(mut self) -> Result<(Offset, Chunk), ChunkError> {
        if let Some(error) = self.error {
            return Err(error);
        }
        let mut label_offsets;
        let mut constants;
        let mut bytes;
        match self.prefix {
            Some(chunk) => {
                label_offsets = chunk.labels;
                constants = chunk.constants;
                bytes = chunk.bytes;
            }
            None => {
                label_offsets = std::collections::HashMap::default();
                constants = vec![];
                bytes = vec![];
            }
        }

        let mut entrypoint = 0;
        let mut distance = bytes.len() as u32;
        for (offset, line) in self.lines.iter_mut().enumerate() {
            if offset == self.entry_line {
                entrypoint = offset as u32 + distance;
            }
            for label in line.labels.drain(..) {
                label_offsets.insert(label, offset as u32 + distance);
            }
            if line.value.is_some() {
                distance += 4;
            }
        }

        for line in self.lines.into_iter() {
            bytes.push(line.opcode as u8);
            match line.value {
                None => {}
                Some(Parameter::Offset(offset)) => bytes.extend(offset.to_be_bytes()),
                Some(Parameter::Label(label)) => {
                    let offset = *label_offsets
                        .get(&label)
                        .ok_or_else(|| ChunkError::MissingLabel(label.to_owned()))?;
                    bytes.extend(offset.to_be_bytes());
                }
                Some(Parameter::Value(value)) => {
                    let index = match constants.iter().position(|val| *val == value) {
                        None => {
                            let index = constants.len() as u32;
                            constants.push(value);
                            index
                        }
                        Some(index) => index as u32,
                    };
                    bytes.extend(index.to_be_bytes());
                }
                Some(Parameter::Reference(label)) => {
                    let offset = *label_offsets
                        .get(&label)
                        .ok_or_else(|| ChunkError::MissingLabel(label.to_owned()))?;
                    let value = Value::Procedure(Procedure::new(offset));
                    let index = match constants.iter().position(|val| *val == value) {
                        None => {
                            let index = constants.len() as u32;
                            constants.push(value);
                            index
                        }
                        Some(index) => index as u32,
                    };
                    bytes.extend(index.to_be_bytes());
                }
            }
        }

        Ok((
            entrypoint,
            Chunk {
                labels: label_offsets,
                constants,
                bytes,
            },
        ))
    }

    /// Parse a string of written ASM.
    ///
    /// Returns a `SyntaxError` if the string is not valid.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// builder
    ///     .parse(r#"
    ///         CONST 1
    ///         CONST 2
    ///         ADD
    ///         EXIT
    ///     "#)
    ///     .unwrap()
    ///     .build()
    ///     .unwrap()
    /// ```
    pub fn parse(&mut self, source: &str) -> &mut Self {
        self.error = self.try_parse(source).err();
        self
    }

    fn try_parse(&mut self, source: &str) -> Result<(), ChunkError> {
        let mut reader = AsmReader::new(source, self.interner.clone());

        while !reader.is_empty() {
            while let Some(label) = reader.label_definition() {
                self.label(label);
            }
            let opcode = reader.opcode().ok_or(ChunkError::InvalidAsm)?;
            match opcode {
                OpCode::Const => {
                    let value = match reader.value().ok_or(ChunkError::InvalidAsm)? {
                        asm::Value::Label(label) => Parameter::Reference(label),
                        asm::Value::Value(value) => Parameter::Value(value),
                    };
                    self.write_line(opcode, Some(value));
                }
                _ => match opcode.params() {
                    0 => {
                        self.write_line(opcode, None);
                    }
                    1 => {
                        let param = match reader.parameter().ok_or(ChunkError::InvalidAsm)? {
                            asm::Parameter::Label(label) => Parameter::Label(label),
                            asm::Parameter::Offset(label) => Parameter::Offset(label),
                        };
                        self.write_line(opcode, Some(param));
                    }
                    _ => unreachable!(),
                },
            }
        }

        Ok(())
    }
}
