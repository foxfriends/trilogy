use super::writer::ProgramWriter;
use crate::bytecode::asm::{AsmContext, AsmError};
use crate::bytecode::{LabelAlreadyInserted, Offset};
use crate::{Atom, Instruction, Program};

#[derive(Default)]
pub struct ProgramBuilder {
    context: AsmContext,
    writer: ProgramWriter,
}

impl ProgramBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Writes the next instruction.
    pub fn write_instruction(&mut self, instruction: Instruction) -> &mut Self {
        self.writer.write_instruction(instruction);
        self
    }

    /// Writes a sequence of instructions all at once.
    pub fn write_instructions(
        &mut self,
        instructions: impl IntoIterator<Item = Instruction>,
    ) -> &mut Self {
        for instruction in instructions {
            self.writer.write_instruction(instruction);
        }
        self
    }

    /// Writes a label at the position of the next instruction in the program. Returns the
    /// offset of that label, or an error if a label with this name has already been set.
    pub fn write_label(&mut self, label: String) -> Result<&mut Self, LabelAlreadyInserted> {
        self.context.insert_label(label)?;
        Ok(self)
    }

    /// Creates an Atom from a string. Atoms are unique within the scope of a program, so
    /// cannot be created externally.
    pub fn atom(&mut self, value: &str) -> Atom {
        self.context.intern(value)
    }

    /// Retrieves the offset of the next instruction to be written.
    pub fn offset(&self) -> Offset {
        self.writer.offset()
    }

    pub fn build(self) -> Result<Program, AsmError> {
        self.writer.finish(self.context)
    }
}
