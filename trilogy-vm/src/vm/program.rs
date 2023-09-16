use super::{Chunk, ChunkBuilder};

pub trait Program {
    fn chunk(&mut self, builder: ChunkBuilder) -> Chunk;
    fn entrypoint(&mut self, builder: ChunkBuilder) -> Chunk;
}
