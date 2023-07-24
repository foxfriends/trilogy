mod labeler;
mod scope;

pub(crate) use labeler::Labeler;
pub(crate) use scope::{Binding, Scope};
use trilogy_ir::Id;
use trilogy_vm::{Atom, Instruction, LabelAlreadyInserted, OpCode, ProgramBuilder, Value};

pub(crate) struct Context<'a> {
    pub labeler: &'a mut Labeler,
    pub scope: Scope<'a>,
    builder: &'a mut ProgramBuilder,
}

impl<'a> Context<'a> {
    pub fn new(
        builder: &'a mut ProgramBuilder,
        labeler: &'a mut Labeler,
        scope: Scope<'a>,
    ) -> Self {
        Self {
            labeler,
            scope,
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

    pub fn shift(&mut self, label: &str) -> &mut Self {
        self.builder
            .write_opcode(OpCode::Shift)
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

    pub fn declare_variables(&mut self, variables: impl Iterator<Item = Id>) {
        for id in variables {
            if self.scope.declare_variable(id) {
                self.write_instruction(Instruction::Const(Value::Unit));
            }
        }
    }

    pub fn undeclare_variables(&mut self, variables: impl Iterator<Item = Id>, pop: bool) {
        for id in variables {
            if self.scope.undeclare_variable(&id) && pop {
                self.write_instruction(Instruction::Pop);
            }
        }
    }
}
