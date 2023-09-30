use super::super::{Instruction, Offset};
use super::Chunk;

pub struct ChunkIter<'a> {
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
    pub fn offsets(self) -> impl Iterator<Item = (Offset, Instruction)> + 'a {
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
