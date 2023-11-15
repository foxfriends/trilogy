use crate::prelude::*;
pub(crate) use trilogy_vm::ChunkWriter;
pub(crate) use trilogy_vm::Instruction;

pub(crate) trait StatefulChunkWriterExt:
    StackTracker + ChunkWriter + LabelMaker + Sized
{
    fn r#continue<S: Into<String>>(&mut self, label: S) -> &mut Self {
        let cont = self.intermediate();
        self.instruction(Instruction::Variable)
            .continuation(|context| {
                // Continue is called with a value that is ignored. This is definitely an oversight
                // that I should get around to fixing... or maybe there's a way to use that value?
                context
                    .unlock_function()
                    .instruction(Instruction::Pop)
                    .jump(label);
            })
            .instruction(Instruction::SetLocal(cont));
        self.end_intermediate();
        self
    }

    fn r#break<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.continuation(|context| {
            // Break is called with a value that is ignored. This is definitely an oversight
            // that I should get around to fixing... or maybe there's a way to use that value?
            context
                .unlock_function()
                .instruction(Instruction::Pop)
                .jump(label);
        })
    }
}

impl<T> StatefulChunkWriterExt for T where T: StackTracker + ChunkWriter + LabelMaker {}
