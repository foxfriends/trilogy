use std::collections::HashMap;

use super::error::ChunkError;
use super::{Chunk, ChunkWriter};
use crate::atom::AtomInterner;
use crate::bytecode::asm::{self, AsmReader};
use crate::callable::Procedure;
use crate::{Atom, Instruction, Offset, OpCode, Value};

#[derive(Eq, PartialEq, Clone)]
enum Entrypoint {
    Line(usize),
    Index(u32),
    Label(String),
}

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
    entrypoint: Entrypoint,
    interner: AtomInterner,
    lines: Vec<Line>,
    current_labels: Vec<String>,
    error: Option<ChunkError>,
}

impl ChunkBuilder {
    pub(crate) fn new(interner: AtomInterner) -> Self {
        Self {
            entrypoint: Entrypoint::Line(0),
            interner,
            lines: vec![],
            current_labels: vec![],
            error: None,
        }
    }

    /// Instantiate an anonymous atom for the current runtime. An anonymous atom
    /// cannot be re-created. The provided label is shown when debugging, but two
    /// atoms with the same label are not the same value.
    pub fn anon_atom(&self, label: &str) -> Atom {
        Atom::new_unique(label.to_owned())
    }
}

impl ChunkBuilder {
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
        self.entrypoint = Entrypoint::Line(self.lines.len());
        self
    }

    /// Sets the entrypoint of this chunk to be at a label previously written.
    pub fn entrypoint_existing(&mut self, label: &str) -> &mut Self {
        self.entrypoint = Entrypoint::Label(label.to_owned());
        self
    }

    /// Parse a string of written ASM.
    ///
    /// If an error is encountered while parsing the string, the builder will be set to
    /// an invalid state, and will cause an `Err` to be returned when calling `build`.
    ///
    /// ```ignore
    /// builder
    ///     .parse(r#"
    ///         CONST 1
    ///         CONST 2
    ///         ADD
    ///         EXIT
    ///     "#)
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
            while let Some(label) = reader.label_definition().map_err(ChunkError::InvalidAsm)? {
                self.label(label);
            }
            let opcode = reader.opcode().map_err(ChunkError::InvalidAsm)?;
            match opcode {
                OpCode::Const | OpCode::Chunk => {
                    let value = match reader.value().map_err(ChunkError::InvalidAsm)? {
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
                        let param = match reader.parameter().map_err(ChunkError::InvalidAsm)? {
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

    /// Construct the [`Chunk`][] that was being built. Fails if any labels were referenced
    /// but not defined.
    pub(crate) fn build(self) -> Result<(Offset, Chunk), ChunkError> {
        let mut chunk = Chunk {
            labels: HashMap::default(),
            constants: vec![],
            bytes: vec![],
        };
        let offset = self.build_from(&mut chunk)?;
        Ok((offset, chunk))
    }

    pub(crate) fn build_from(mut self, chunk: &mut Chunk) -> Result<Offset, ChunkError> {
        if let Some(error) = self.error {
            return Err(error);
        }

        let mut distance = chunk.bytes.len() as u32;
        for (i, line) in self.lines.iter_mut().enumerate() {
            if Entrypoint::Line(i) == self.entrypoint {
                self.entrypoint = Entrypoint::Index(i as u32 + distance);
            }
            for label in line.labels.drain(..) {
                chunk.labels.insert(label, i as u32 + distance);
            }
            if line.value.is_some() {
                distance += 4;
            }
        }

        for line in self.lines.into_iter() {
            chunk.bytes.push(line.opcode as u8);
            match line.value {
                None => {}
                Some(Parameter::Offset(offset)) => chunk.bytes.extend(offset.to_be_bytes()),
                Some(Parameter::Label(label)) => {
                    let offset = *chunk
                        .labels
                        .get(&label)
                        .ok_or_else(|| ChunkError::MissingLabel(label.to_owned()))?;
                    chunk.bytes.extend(offset.to_be_bytes());
                }
                Some(Parameter::Value(value)) => {
                    let index = match chunk.constants.iter().position(|val| *val == value) {
                        None => {
                            let index = chunk.constants.len() as u32;
                            chunk.constants.push(value);
                            index
                        }
                        Some(index) => index as u32,
                    };
                    chunk.bytes.extend(index.to_be_bytes());
                }
                Some(Parameter::Reference(label)) => {
                    let offset = *chunk
                        .labels
                        .get(&label)
                        .ok_or_else(|| ChunkError::MissingLabel(label.to_owned()))?;
                    let value = Value::from(Procedure::new(offset));
                    let index = match chunk.constants.iter().position(|val| *val == value) {
                        None => {
                            let index = chunk.constants.len() as u32;
                            chunk.constants.push(value);
                            index
                        }
                        Some(index) => index as u32,
                    };
                    chunk.bytes.extend(index.to_be_bytes());
                }
            }
        }

        let entry = match self.entrypoint {
            Entrypoint::Line(..) => 0,
            Entrypoint::Label(label) => chunk
                .labels
                .get(&label)
                .copied()
                .ok_or(ChunkError::MissingLabel(label))?,
            Entrypoint::Index(index) => index,
        };
        Ok(entry)
    }
}

impl ChunkWriter for ChunkBuilder {
    /// Instantiate an atom for the current runtime. Atoms cannot be created except
    /// for within the context of a particular runtime's global atom table.
    fn make_atom<S: AsRef<str>>(&self, atom: S) -> Atom {
        self.interner.intern(atom.as_ref())
    }

    fn label<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.current_labels.push(label.into());
        self
    }

    /// Insert a CONST instruction that references a procedure located at the
    /// given label.
    ///
    /// ```asm
    /// CONST &label
    /// ```
    fn reference<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.write_line(OpCode::Const, Some(Parameter::Reference(label.into())))
    }

    /// Insert a JUMP instruction to a given label.
    ///
    /// ```asm
    /// JUMP &label
    /// ```
    fn jump<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.write_line(OpCode::Jump, Some(Parameter::Label(label.into())))
    }

    /// Insert a JUMPF instruction to a given label.
    ///
    /// ```asm
    /// JUMPF &label
    /// ```
    fn cond_jump<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.write_line(OpCode::CondJump, Some(Parameter::Label(label.into())))
    }

    /// Insert a CLOSE instruction to a given label.
    ///
    /// ```asm
    /// CLOSE &label
    /// ```
    fn close<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.write_line(OpCode::Close, Some(Parameter::Label(label.into())))
    }

    /// Insert a SHIFT instruction to a given label.
    ///
    /// ```asm
    /// SHIFT &label
    /// ```
    fn shift<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.write_line(OpCode::Shift, Some(Parameter::Label(label.into())))
    }

    /// Insert an instruction.
    ///
    /// All labels currently in the buffer will be assigned to this line, and
    /// the buffer will be cleared.
    fn instruction(&mut self, instruction: Instruction) -> &mut Self {
        let opcode = instruction.op_code();
        let value = match instruction {
            Instruction::Const(value) => Some(Parameter::Value(value)),
            Instruction::Chunk(value) => Some(Parameter::Value(value)),
            Instruction::LoadLocal(offset) => Some(Parameter::Offset(offset)),
            Instruction::SetLocal(offset) => Some(Parameter::Offset(offset)),
            Instruction::InitLocal(offset) => Some(Parameter::Offset(offset)),
            Instruction::UnsetLocal(offset) => Some(Parameter::Offset(offset)),
            Instruction::IsSetLocal(offset) => Some(Parameter::Offset(offset)),
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
}
