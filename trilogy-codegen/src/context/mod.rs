mod labeler;
mod scope;

pub(crate) use labeler::Labeler;
pub(crate) use scope::{Binding, Scope};
use trilogy_vm::{Atom, Instruction, LabelAlreadyInserted, OpCode, ProgramBuilder};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum KwReturn {
    Return,
    Reset,
}

pub(crate) struct Context<'a> {
    pub labeler: Labeler,
    pub scope: Scope,
    builder: &'a mut ProgramBuilder,

    kw_return: KwReturn,
    #[allow(dead_code)]
    kw_resume: Option<usize>,
    #[allow(dead_code)]
    kw_cancel: Option<usize>,
    #[allow(dead_code)]
    kw_break: Option<usize>,
    #[allow(dead_code)]
    kw_continue: Option<usize>,
}

impl<'a> Context<'a> {
    pub fn new(builder: &'a mut ProgramBuilder, location: String) -> Self {
        Self {
            labeler: Labeler::new(location),
            scope: Scope::default(),
            builder,
            kw_return: KwReturn::Return,
            kw_resume: None,
            kw_cancel: None,
            kw_break: None,
            kw_continue: None,
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
        self.kw_return = KwReturn::Reset;
        self
    }

    pub fn reset(&mut self) -> &mut Self {
        self.kw_return = KwReturn::Return;
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

    pub fn kw_return(&self) -> Instruction {
        match self.kw_return {
            KwReturn::Reset => Instruction::Reset,
            KwReturn::Return => Instruction::Return,
        }
    }
}
