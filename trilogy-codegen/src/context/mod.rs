mod labeler;
mod scope;

pub(crate) use labeler::Labeler;
pub(crate) use scope::{Binding, Scope};
use trilogy_ir::Id;
use trilogy_vm::{Atom, ChunkBuilder, Instruction};

pub(crate) struct Context<'a> {
    pub labeler: &'a mut Labeler,
    pub scope: Scope<'a>,
    builder: &'a mut ChunkBuilder,
}

impl<'a> Context<'a> {
    pub fn new(builder: &'a mut ChunkBuilder, labeler: &'a mut Labeler, scope: Scope<'a>) -> Self {
        Self {
            labeler,
            scope,
            builder,
        }
    }

    pub fn write_procedure_reference(&mut self, label: impl Into<String>) -> &mut Self {
        self.builder.reference(label);
        self
    }

    pub fn cond_jump(&mut self, label: impl Into<String>) -> &mut Self {
        self.builder.cond_jump(label);
        self
    }

    pub fn jump(&mut self, label: impl Into<String>) -> &mut Self {
        self.builder.jump(label);
        self
    }

    pub fn shift(&mut self, label: impl Into<String>) -> &mut Self {
        self.builder.shift(label);
        self
    }

    pub fn close(&mut self, label: impl Into<String>) -> &mut Self {
        self.builder.close(label);
        self
    }

    pub fn instruction(&mut self, instruction: Instruction) -> &mut Self {
        self.builder.instruction(instruction);
        self
    }

    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.builder.label(label);
        self
    }

    pub fn atom(&mut self, value: &str) -> Atom {
        self.builder.atom(value)
    }

    pub fn declare_variables(&mut self, variables: impl IntoIterator<Item = Id>) -> usize {
        let mut n = 0;
        for id in variables {
            if self.scope.declare_variable(id.clone()) {
                let label = self.labeler.var(&id);
                self.label(label);
                self.instruction(Instruction::Variable);
                n += 1;
            }
        }
        n
    }

    pub fn undeclare_variables(&mut self, variables: impl IntoIterator<Item = Id>, pop: bool) {
        for id in variables {
            if self.scope.undeclare_variable(&id) && pop {
                self.instruction(Instruction::Pop);
            }
        }
    }
}
