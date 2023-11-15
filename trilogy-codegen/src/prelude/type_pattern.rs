use super::LabelMaker;
use trilogy_vm::{ChunkWriter, Instruction};

pub(crate) trait TypePattern {
    fn write<W: ChunkWriter + LabelMaker>(&self, writer: &mut W, destination: Result<&str, &str>);
}

impl TypePattern for () {
    fn write<W: ChunkWriter + LabelMaker>(
        &self,
        _writer: &mut W,
        _destination: Result<&str, &str>,
    ) {
    }
}

impl TypePattern for str {
    fn write<W: ChunkWriter + LabelMaker>(&self, writer: &mut W, destination: Result<&str, &str>) {
        writer
            .instruction(Instruction::Copy)
            .instruction(Instruction::TypeOf)
            .atom(self);
        match destination {
            Ok(destination) => writer
                .instruction(Instruction::ValNeq)
                .cond_jump(destination),
            Err(destination) => writer
                .instruction(Instruction::ValEq)
                .cond_jump(destination),
        };
    }
}

impl TypePattern for [&str] {
    fn write<W: ChunkWriter + LabelMaker>(&self, writer: &mut W, destination: Result<&str, &str>) {
        match destination {
            Ok(destination) => {
                let done = writer.make_label("done");
                for t in self {
                    t.write(writer, Err(&done));
                }
                writer.jump(destination).label(done);
            }
            Err(destination) => {
                let done = writer.make_label("done");
                for t in self {
                    t.write(writer, Ok(&done));
                }
                writer.jump(destination).label(done);
            }
        }
    }
}
