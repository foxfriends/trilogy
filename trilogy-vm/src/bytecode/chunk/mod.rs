use crate::{Instruction, Offset, OpCode, Value};
use std::collections::HashMap;
use std::fmt::{self, Display};

mod builder;

pub use builder::{ChunkBuilder, ChunkError};

/// A chunk of independently compiled source code for this VM.
#[derive(Clone, Debug)]
pub(crate) struct Chunk {
    labels: HashMap<String, u32>,
    pub constants: Vec<Value>,
    pub bytes: Vec<u8>,
}

impl Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (offset, instruction) in self.into_iter().offsets() {
            let labels = self
                .labels
                .iter()
                .filter(|&(.., &label_offset)| label_offset == offset);
            for (label, ..) in labels {
                writeln!(f, "{label}:")?;
            }

            "    ".fmt(f)?;
            match &instruction {
                Instruction::Const(Value::Procedure(procedure)) => {
                    let offset = procedure.ip();
                    match self.labels.iter().find(|(.., pos)| **pos == offset) {
                        Some((label, ..)) => writeln!(f, "{} &{}", instruction.op_code(), label)?,
                        _ => writeln!(f, "{instruction}")?,
                    }
                }
                Instruction::Jump(offset)
                | Instruction::CondJump(offset)
                | Instruction::Shift(offset)
                | Instruction::Close(offset) => {
                    match self.labels.iter().find(|(.., pos)| *pos == offset) {
                        Some((label, ..)) => writeln!(f, "{} &{}", instruction.op_code(), label)?,
                        _ => writeln!(f, "{instruction}")?,
                    }
                }
                _ => writeln!(f, "{instruction}")?,
            }
        }

        Ok(())
    }
}

impl Chunk {
    pub fn opcode(&self, offset: Offset) -> OpCode {
        OpCode::try_from(self.bytes[offset as usize]).unwrap()
    }

    pub fn offset(&self, offset: Offset) -> Offset {
        Offset::from_be_bytes(
            self.bytes[offset as usize..offset as usize + 4]
                .try_into()
                .unwrap(),
        )
    }

    pub fn constant(&self, offset: Offset) -> Value {
        let index = self.offset(offset);
        self.constants[index as usize].clone()
    }
}

pub(crate) struct ChunkIter<'a> {
    offset: usize,
    chunk: &'a Chunk,
}

impl<'a> IntoIterator for &'a Chunk {
    type Item = Instruction;
    type IntoIter = ChunkIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ChunkIter {
            offset: 0,
            chunk: self,
        }
    }
}

impl Iterator for ChunkIter<'_> {
    type Item = Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset == self.chunk.bytes.len() {
            return None;
        }
        let instruction = Instruction::from_chunk(self.chunk, self.offset as u32);
        self.offset += instruction.byte_len();
        Some(instruction)
    }
}

impl<'a> ChunkIter<'a> {
    fn offsets(self) -> impl Iterator<Item = (Offset, Instruction)> + 'a {
        struct Offsets<'a> {
            chunks: ChunkIter<'a>,
        }

        impl Iterator for Offsets<'_> {
            type Item = (Offset, Instruction);

            fn next(&mut self) -> Option<Self::Item> {
                let offset = self.chunks.offset;
                let next = self.chunks.next()?;
                Some((offset as u32, next))
            }
        }

        Offsets { chunks: self }
    }
}
