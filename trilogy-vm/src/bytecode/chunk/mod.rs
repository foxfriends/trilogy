use crate::callable::{Callable, CallableKind};
use crate::{Instruction, Offset, OpCode, Value};
use std::collections::HashMap;
use std::fmt::{self, Debug, Display};

mod builder;
mod error;
mod iter;

pub use builder::ChunkBuilder;
pub use error::ChunkError;
pub use iter::ChunkIter;

/// A chunk of independently compiled source code for the Trilogy VM.
///
/// It is currently not defined as to what happens if you run a chunk on a
/// [`VirtualMachine`][crate::VirtualMachine] instance that is not the one that compiled it.
#[derive(Clone)]
pub struct Chunk {
    labels: HashMap<String, u32>,
    pub(crate) constants: Vec<Value>,
    pub(crate) bytes: Vec<u8>,
}

impl Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (offset, instruction) in self.into_iter().offsets() {
            let labels = self
                .labels
                .iter()
                .filter(|&(.., &label_offset)| label_offset == offset);
            for (label, ..) in labels {
                writeln!(f, "{label:?}:")?;
            }

            write!(f, "    ")?;
            match &instruction {
                Instruction::Const(Value::Callable(Callable(CallableKind::Procedure(
                    procedure,
                )))) => {
                    let offset = procedure.ip();
                    match self.labels.iter().find(|(.., pos)| **pos == offset) {
                        Some((label, ..)) => {
                            writeln!(f, "{: <6} &{:?}", instruction.op_code(), label)?
                        }
                        _ => writeln!(f, "{instruction}")?,
                    }
                }
                Instruction::Jump(offset)
                | Instruction::CondJump(offset)
                | Instruction::Shift(offset)
                | Instruction::Close(offset) => {
                    match self.labels.iter().find(|(.., pos)| *pos == offset) {
                        Some((label, ..)) => {
                            writeln!(f, "{: <6} &{:?}", instruction.op_code(), label)?
                        }
                        _ => writeln!(f, "{instruction}")?,
                    }
                }
                _ => writeln!(f, "{instruction}")?,
            }
        }

        Ok(())
    }
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut print_offset = 0;
        for (offset, instruction) in self.into_iter().offsets() {
            let labels = self
                .labels
                .iter()
                .filter(|&(.., &label_offset)| label_offset == offset);
            for (label, ..) in labels {
                writeln!(f, "{label:?}:")?;
            }

            write!(f, "{:>8}: ", print_offset)?;
            match &instruction {
                Instruction::Const(Value::Callable(Callable(CallableKind::Procedure(
                    procedure,
                )))) => {
                    let offset = procedure.ip();
                    match self.labels.iter().find(|(.., pos)| **pos == offset) {
                        Some((label, ..)) => {
                            writeln!(f, "{: <6} &{:?}", instruction.op_code(), label)?
                        }
                        _ => writeln!(f, "{instruction}")?,
                    }
                }
                Instruction::Jump(offset)
                | Instruction::CondJump(offset)
                | Instruction::Shift(offset)
                | Instruction::Close(offset) => {
                    match self.labels.iter().find(|(.., pos)| *pos == offset) {
                        Some((label, ..)) => {
                            writeln!(f, "{: <6} &{:?}", instruction.op_code(), label)?
                        }
                        _ => writeln!(f, "{instruction}")?,
                    }
                }
                _ => writeln!(f, "{instruction}")?,
            }
            print_offset += instruction.byte_len();
        }

        Ok(())
    }
}

impl Chunk {
    pub(crate) fn opcode(&self, offset: Offset) -> OpCode {
        OpCode::try_from(self.bytes[offset as usize]).unwrap()
    }

    pub(crate) fn offset(&self, offset: Offset) -> Offset {
        Offset::from_be_bytes(
            self.bytes[offset as usize..offset as usize + 4]
                .try_into()
                .unwrap(),
        )
    }

    pub(crate) fn constant(&self, offset: Offset) -> Value {
        let index = self.offset(offset);
        self.constants[index as usize].clone()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = Instruction> + '_ {
        self.into_iter()
    }
}
