use super::instruction::{Instruction, RawInstruction};
use crate::callable::{Callable, CallableKind};
use crate::{Offset, Value};
use std::collections::HashMap;
use std::fmt::{self, Debug, Display};

mod annotation;
mod builder;
mod error;
mod iter;
mod line;
#[macro_use]
mod writer;

pub use annotation::{Annotation, Location, Note};
pub use builder::ChunkBuilder;
pub use error::ChunkError;
pub(crate) use line::{Line, Parameter};
pub use writer::ChunkWriter;

/// A chunk of independently compiled source code for the Trilogy VM.
///
/// It is currently not defined as to what happens if you run a chunk on a
/// [`VirtualMachine`][crate::VirtualMachine] instance that is not the one that compiled it.
#[derive(Clone)]
pub struct Chunk {
    pub(crate) annotations: Vec<Annotation>,
    pub(crate) labels: HashMap<String, Offset>,
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
    #[inline(always)]
    #[allow(clippy::unnecessary_cast)]
    pub(crate) fn instruction_bytes(&self, index: Offset) -> RawInstruction {
        let mut bytes = [0; std::mem::size_of::<RawInstruction>()];
        bytes.copy_from_slice(
            &self.bytes[index as usize..index as usize + std::mem::size_of::<RawInstruction>()],
        );
        unsafe { std::mem::transmute(bytes) }
    }

    #[inline(always)]
    #[allow(clippy::unnecessary_cast)]
    pub(crate) fn constant(&self, index: Offset) -> Value {
        self.constants[index as usize].clone()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = Instruction> + '_ {
        self.into_iter()
    }

    pub(crate) fn get_annotations(&self, ip: Offset) -> Vec<Annotation> {
        self.annotations
            .iter()
            .filter(|note| note.spans(ip))
            .cloned()
            .collect()
    }
}
