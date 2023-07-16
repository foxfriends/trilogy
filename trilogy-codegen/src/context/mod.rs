mod labeler;
mod scope;

pub(crate) use labeler::Labeler;
pub(crate) use scope::{Binding, Scope};
use trilogy_vm::{Atom, Instruction, LabelAlreadyInserted, OpCode, ProgramBuilder};

pub(crate) struct Context<'a> {
    pub labeler: Labeler,
    pub scope: Scope,
    builder: &'a mut ProgramBuilder,
    pub stack_height: usize,
}

impl<'a> Context<'a> {
    pub fn new(builder: &'a mut ProgramBuilder, location: String) -> Self {
        Self {
            labeler: Labeler::new(location),
            scope: Scope::default(),
            builder,
            stack_height: 0,
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
        self.stack_height -= 1;
        self
    }

    pub fn write_instruction(&mut self, instruction: Instruction) -> &mut Self {
        use Instruction::*;
        match instruction {
            Const(..) => self.stack_height += 1,
            Load => {}
            Set => self.stack_height -= 2,
            Alloc => {}
            Free => self.stack_height -= 1,
            LoadRegister(..) => self.stack_height += 1,
            SetRegister(..) => self.stack_height -= 1,
            Copy => self.stack_height += 1,
            Pop => self.stack_height -= 1,
            Swap => {}
            Add => self.stack_height -= 1,
            Subtract => self.stack_height -= 1,
            Multiply => self.stack_height -= 1,
            Divide => self.stack_height -= 1,
            Remainder => self.stack_height -= 1,
            IntDivide => self.stack_height -= 1,
            Power => self.stack_height -= 1,
            Negate => self.stack_height -= 1,
            Glue => self.stack_height -= 1,
            Access => self.stack_height -= 1,
            Assign => self.stack_height -= 3,
            Not => {}
            And => self.stack_height -= 1,
            Or => self.stack_height -= 1,
            BitwiseAnd => self.stack_height -= 1,
            BitwiseOr => self.stack_height -= 1,
            BitwiseXor => self.stack_height -= 1,
            BitwiseNeg => {}
            LeftShift => self.stack_height -= 1,
            RightShift => self.stack_height -= 1,
            Cons => self.stack_height -= 1,
            Uncons => self.stack_height += 1,
            First => {}
            Second => {}
            Construct => self.stack_height -= 1,
            Destruct => self.stack_height += 1,
            Leq => self.stack_height -= 1,
            Lt => self.stack_height -= 1,
            Geq => self.stack_height -= 1,
            Gt => self.stack_height -= 1,
            RefEq => self.stack_height -= 1,
            ValEq => self.stack_height -= 1,
            RefNeq => self.stack_height -= 1,
            ValNeq => self.stack_height -= 1,
            Call(arity) => self.stack_height -= arity,
            Return => self.stack_height = 0, // Not relevant
            Shift(..) => self.stack_height += 1,
            Reset => self.stack_height = 0, // Not relevant
            Jump(..) => {}
            JumpBack(..) => {}
            CondJump(..) => self.stack_height -= 1,
            CondJumpBack(..) => self.stack_height -= 1,
            Branch => {}
            Fizzle => {}
            Exit => {}
        };
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
