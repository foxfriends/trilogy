use trilogy_vm::{ChunkBuilder, Program, Value};

pub struct StaticProgram(pub &'static str);

impl Program for StaticProgram {
    fn chunk(&mut self, _input: &Value, _builder: &mut ChunkBuilder) {
        panic!("source string programs do not support external modules");
    }

    fn entrypoint(&mut self, builder: &mut ChunkBuilder) {
        builder.parse(self.0);
    }
}
