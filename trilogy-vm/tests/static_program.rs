use trilogy_vm::{Chunk, ChunkBuilder, Program, Value};

pub struct StaticProgram(pub &'static str);

impl Program for StaticProgram {
    fn chunk(&mut self, _input: Value, _builder: ChunkBuilder) -> Chunk {
        panic!("source string programs do not support external modules");
    }

    fn entrypoint(&mut self, mut builder: ChunkBuilder) -> Chunk {
        builder.parse(self.0).unwrap();
        builder.build().unwrap()
    }
}
