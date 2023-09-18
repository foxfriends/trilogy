use super::Chunk;
use crate::bytecode::asm::{self, AsmReader};
use crate::Procedure;
use crate::{atom::AtomInterner, Atom, Instruction, OpCode, Value};

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

/// Builder for constructing a [`Chunk`][].
pub struct ChunkBuilder {
    interner: AtomInterner,
    lines: Vec<Line>,
    current_labels: Vec<String>,
}

/// An error that can occur when building a Chunk incorrectly.
#[derive(Debug)]
pub enum ChunkError {
    /// A referenced label was not defined.
    MissingLabel(String),
}

/// Indicates that there was a syntax error in the ASM string.
#[derive(Debug)]
pub struct SyntaxError;

impl ChunkBuilder {
    pub(crate) fn new(interner: AtomInterner) -> Self {
        Self {
            interner,
            lines: vec![],
            current_labels: vec![],
        }
    }

    /// Instantiate an atom for the current runtime. Atoms cannot be created except
    /// for within the context of a particular runtime's global atom table.
    pub fn atom(&mut self, atom: &str) -> Atom {
        self.interner.intern(atom)
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

    pub(super) fn write_line(&mut self, opcode: OpCode, value: Option<Parameter>) -> &mut Self {
        let labels = self.current_labels.drain(..).collect();
        self.lines.push(Line {
            labels,
            opcode,
            value,
        });
        self
    }

    /// Construct the [`Chunk`][] that was being built. Fails if any labels were referenced
    /// but not defined.
    pub fn build(mut self) -> Result<Chunk, ChunkError> {
        let mut label_offsets = std::collections::HashMap::new();
        let mut constants = vec![];
        let mut bytes = vec![];

        let mut params = 0;
        for (offset, line) in self.lines.iter_mut().enumerate() {
            for label in line.labels.drain(..) {
                label_offsets.insert(label, offset as u32 + params);
            }
            if line.value.is_some() {
                params += 4;
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

        Ok(Chunk {
            labels: label_offsets,
            constants,
            bytes,
        })
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
    pub fn parse(&mut self, source: &str) -> Result<&mut Self, SyntaxError> {
        let mut reader = AsmReader::new(source, self.interner.clone());

        while !reader.is_empty() {
            while let Some(label) = reader.label_definition() {
                self.label(label);
            }
            let opcode = reader.opcode().ok_or(SyntaxError)?;
            match opcode {
                OpCode::Const => {
                    let value = match reader.value().ok_or(SyntaxError)? {
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
                        let param = match reader.parameter().ok_or(SyntaxError)? {
                            asm::Parameter::Label(label) => Parameter::Label(label),
                            asm::Parameter::Offset(label) => Parameter::Offset(label),
                        };
                        self.write_line(opcode, Some(param));
                    }
                    _ => unreachable!(),
                },
            }
        }

        Ok(self)
    }
}
