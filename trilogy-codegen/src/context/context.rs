use super::{Labeler, Scope};
use crate::prelude::*;
use trilogy_ir::Id;
use trilogy_vm::{Atom, ChunkBuilder, ChunkWriter, Instruction, Offset, Value};

pub(crate) struct Context<'a> {
    labeler: &'a mut Labeler,
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
}

impl ChunkWriter for Context<'_> {
    fn reference<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.builder.reference(label);
        self
    }

    fn cond_jump<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.builder.cond_jump(label);
        self
    }

    fn jump<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.builder.jump(label);
        self
    }

    fn shift<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.builder.shift(label);
        self
    }

    fn close<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.builder.close(label);
        self
    }

    fn instruction(&mut self, instruction: Instruction) -> &mut Self {
        self.builder.instruction(instruction);
        self
    }

    fn label<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.builder.label(label);
        self
    }

    fn constant<V: Into<Value>>(&mut self, value: V) -> &mut Self {
        self.builder.constant(value);
        self
    }

    fn make_atom<S: AsRef<str>>(&self, value: S) -> Atom {
        self.builder.make_atom(value)
    }
}

impl LabelMaker for Context<'_> {
    fn make_label(&mut self, label: &str) -> String {
        self.labeler.unique_hint(label)
    }
}

impl StackTracker for Context<'_> {
    fn intermediate(&mut self) -> Offset {
        self.scope.intermediate()
    }

    fn end_intermediate(&mut self) -> &mut Self {
        self.scope.end_intermediate();
        self
    }

    fn push_continue(&mut self, offset: Offset) -> &mut Self {
        self.scope.push_continue(offset);
        self
    }

    fn pop_continue(&mut self) -> &mut Self {
        self.scope.pop_continue();
        self
    }

    fn push_break(&mut self, offset: Offset) -> &mut Self {
        self.scope.push_break(offset);
        self
    }

    fn pop_break(&mut self) -> &mut Self {
        self.scope.pop_break();
        self
    }

    fn push_cancel(&mut self, offset: Offset) -> &mut Self {
        self.scope.push_cancel(offset);
        self
    }

    fn pop_cancel(&mut self) -> &mut Self {
        self.scope.pop_cancel();
        self
    }

    fn push_resume(&mut self, offset: Offset) -> &mut Self {
        self.scope.push_resume(offset);
        self
    }

    fn pop_resume(&mut self) -> &mut Self {
        self.scope.pop_resume();
        self
    }
}

impl Context<'_> {
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
                let label = self.labeler.unvar(&id);
                self.label(label);
                self.instruction(Instruction::Pop);
            }
        }
    }
}
