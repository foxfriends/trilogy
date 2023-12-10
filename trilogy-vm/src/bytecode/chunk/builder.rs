use super::error::ChunkError;
use super::line::{Line, Parameter};
use super::{Chunk, ChunkWriter};
use crate::atom::AtomInterner;
use crate::bytecode::asm::{self, AsmReader};
use crate::bytecode::optimization::{optimize, LineAdjuster};
use crate::callable::Procedure;
use crate::{Annotation, Atom, Instruction, Offset, OpCode, Value};
use std::collections::HashMap;

#[derive(Clone, Eq, PartialEq)]
enum Entrypoint {
    Offset(Offset),
    Label(String),
}

/// Builder for constructing a chunk of bytecode for the [`VirtualMachine`][crate::VirtualMachine]
/// to execute.
pub struct ChunkBuilder {
    protected_labels: Vec<String>,
    entrypoint: Entrypoint,
    interner: AtomInterner,
    lines: Vec<Line>,
    current_labels: Vec<String>,
    annotations: Vec<Annotation>,
    error: Option<ChunkError>,
    ip: Offset,
}

impl ChunkBuilder {
    pub(crate) fn new(interner: AtomInterner) -> Self {
        Self {
            protected_labels: vec![],
            entrypoint: Entrypoint::Offset(0),
            interner,
            lines: vec![],
            current_labels: vec![],
            annotations: vec![],
            error: None,
            ip: 0,
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
        let line = Line {
            labels,
            opcode,
            value,
        };
        self.ip += line.byte_len() as Offset;
        self.lines.push(line);
        self
    }

    /// Sets the entrypoint of this chunk to be at the line about to be written.
    ///
    /// By default, the entrypoint is the start of the chunk, but this option may
    /// be used to start code execution from some point in the middle.
    pub fn entrypoint(&mut self) -> &mut Self {
        self.entrypoint = Entrypoint::Offset(self.ip);
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
            annotations: vec![],
            constants: vec![],
            bytes: vec![],
        };
        let offset = self.build_from(&mut chunk)?;
        Ok((offset, chunk))
    }

    pub(crate) fn build_from(self, chunk: &mut Chunk) -> Result<Offset, ChunkError> {
        let initial_ip = chunk.bytes.len() as Offset;
        if let Some(error) = self.error {
            return Err(error);
        }

        let mut lines = LineAdjuster::new(self.lines, self.annotations);
        match self.entrypoint {
            Entrypoint::Offset(index) => optimize(&mut lines, Some(index), &self.protected_labels),
            _ => optimize(&mut lines, None, &self.protected_labels),
        };
        let (mut lines, mut annotations) = lines.finish();
        for annotation in &mut annotations {
            annotation.start += initial_ip;
            annotation.end += initial_ip;
        }
        chunk.annotations.append(&mut annotations);

        let mut distance = initial_ip;
        for line in lines.iter_mut() {
            for label in line.labels.drain(..) {
                chunk.labels.insert(label, distance);
            }
            distance += line.byte_len() as Offset;
        }

        for line in lines.into_iter() {
            chunk.bytes.extend((line.opcode as Offset).to_be_bytes());
            match line.value {
                None => chunk.bytes.extend([0; std::mem::size_of::<Offset>()]),
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
                            let index = chunk.constants.len() as Offset;
                            chunk.constants.push(value);
                            index
                        }
                        Some(index) => index as Offset,
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
                            let index = chunk.constants.len() as Offset;
                            chunk.constants.push(value);
                            index
                        }
                        Some(index) => index as Offset,
                    };
                    chunk.bytes.extend(index.to_be_bytes());
                }
            }
        }

        let entry = match self.entrypoint {
            Entrypoint::Offset(offset) => initial_ip + offset,
            Entrypoint::Label(label) => chunk
                .labels
                .get(&label)
                .copied()
                .ok_or(ChunkError::MissingLabel(label))?,
        };
        Ok(entry)
    }
}

impl ChunkWriter for ChunkBuilder {
    fn ip(&self) -> Offset {
        self.ip
    }

    fn annotate(&mut self, annotation: Annotation) -> &mut Self {
        self.annotations.push(annotation);
        self
    }

    fn make_atom<S: AsRef<str>>(&self, atom: S) -> Atom {
        self.interner.intern(atom.as_ref())
    }

    fn label<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.current_labels.push(label.into());
        self
    }

    fn protect(&mut self) -> &mut Self {
        self.protected_labels.extend(self.current_labels.clone());
        self
    }

    fn reference<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.write_line(OpCode::Const, Some(Parameter::Reference(label.into())))
    }

    fn jump<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.write_line(OpCode::Jump, Some(Parameter::Label(label.into())))
    }

    fn cond_jump<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.write_line(OpCode::CondJump, Some(Parameter::Label(label.into())))
    }

    fn close<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.write_line(OpCode::Close, Some(Parameter::Label(label.into())))
    }

    fn shift<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.write_line(OpCode::Shift, Some(Parameter::Label(label.into())))
    }

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
            _ => None,
        };
        self.write_line(opcode, value)
    }
}
