mod labeler;
mod scope;

pub(crate) use labeler::Labeler;
pub(crate) use scope::{Binding, Scope};
use trilogy_vm::{Atom, Instruction, LabelAlreadyInserted, OpCode, ProgramBuilder};

pub(crate) struct Context<'a> {
    pub labeler: Labeler,
    pub scope: Scope,
    builder: &'a mut ProgramBuilder,
}

impl<'a> Context<'a> {
    pub fn new(builder: &'a mut ProgramBuilder, location: String) -> Self {
        Self {
            labeler: Labeler::new(location),
            scope: Scope::default(),
            builder,
        }
    }

    pub fn write_procedure_reference(&mut self, label: String) -> &mut Self {
        let constant = self.builder.store_label(label);
        self.builder.write_reuse_constant(constant);
        self
    }

    pub fn cond_jump(&mut self, label: &str) -> &mut Self {
        self.builder
            .write_opcode(OpCode::CondJump)
            .write_offset_label(label.to_owned());
        self
    }

    pub fn jump(&mut self, label: &str) -> &mut Self {
        self.builder
            .write_opcode(OpCode::Jump)
            .write_offset_label(label.to_owned());
        self
    }

    pub fn write_instruction(&mut self, instruction: Instruction) -> &mut Self {
        self.builder.write_instruction(instruction);
        self
    }

    pub fn write_label(&mut self, label: String) -> Result<&mut Self, LabelAlreadyInserted> {
        self.builder.write_label(label)?;
        Ok(self)
    }

    pub fn atom(&mut self, value: &str) -> Atom {
        self.builder.atom(value)
    }
}
