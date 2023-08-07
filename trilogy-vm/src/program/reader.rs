use super::Program;
use crate::bytecode::{Offset, OpCode};
use crate::Instruction;

pub struct InvalidBytecode;

pub struct ProgramReader<'a> {
    pub(super) program: &'a Program,
    pub(super) ip: usize,
}

impl ProgramReader<'_> {
    fn read_opcode(&mut self) -> Result<OpCode, InvalidBytecode> {
        let instruction = self.program.instructions[self.ip]
            .try_into()
            .map_err(|_| InvalidBytecode)?;
        self.ip += 1;
        Ok(instruction)
    }

    fn read_offset(&mut self) -> Result<Offset, InvalidBytecode> {
        let value = u32::from_be_bytes(
            self.program.instructions[self.ip..self.ip + 4]
                .try_into()
                .map_err(|_| InvalidBytecode)?,
        );
        self.ip += 4;
        Ok(value as usize)
    }

    fn read_instruction(&mut self) -> Result<Instruction, InvalidBytecode> {
        match self.read_opcode()? {
            OpCode::Const => {
                let offset = self.read_offset()?;
                let value = self
                    .program
                    .constants
                    .get(offset)
                    .ok_or(InvalidBytecode)?
                    .clone();
                Ok(Instruction::Const(value))
            }
            OpCode::Load => Ok(Instruction::Load),
            OpCode::Set => Ok(Instruction::Set),
            OpCode::Alloc => Ok(Instruction::Alloc),
            OpCode::Free => Ok(Instruction::Free),
            OpCode::LoadLocal => Ok(Instruction::LoadLocal(self.read_offset()?)),
            OpCode::Variable => Ok(Instruction::Variable),
            OpCode::SetLocal => Ok(Instruction::SetLocal(self.read_offset()?)),
            OpCode::UnsetLocal => Ok(Instruction::UnsetLocal(self.read_offset()?)),
            OpCode::InitLocal => Ok(Instruction::InitLocal(self.read_offset()?)),
            OpCode::LoadRegister => Ok(Instruction::LoadRegister(self.read_offset()?)),
            OpCode::SetRegister => Ok(Instruction::SetRegister(self.read_offset()?)),
            OpCode::Pop => Ok(Instruction::Pop),
            OpCode::Swap => Ok(Instruction::Swap),
            OpCode::TypeOf => Ok(Instruction::TypeOf),
            OpCode::Add => Ok(Instruction::Add),
            OpCode::Copy => Ok(Instruction::Copy),
            OpCode::Clone => Ok(Instruction::Clone),
            OpCode::Subtract => Ok(Instruction::Subtract),
            OpCode::Multiply => Ok(Instruction::Multiply),
            OpCode::Divide => Ok(Instruction::Divide),
            OpCode::Remainder => Ok(Instruction::Remainder),
            OpCode::IntDivide => Ok(Instruction::IntDivide),
            OpCode::Power => Ok(Instruction::Power),
            OpCode::Negate => Ok(Instruction::Negate),
            OpCode::Glue => Ok(Instruction::Glue),
            OpCode::Access => Ok(Instruction::Access),
            OpCode::Assign => Ok(Instruction::Assign),
            OpCode::Insert => Ok(Instruction::Insert),
            OpCode::Delete => Ok(Instruction::Delete),
            OpCode::Contains => Ok(Instruction::Contains),
            OpCode::Entries => Ok(Instruction::Entries),
            OpCode::Skip => Ok(Instruction::Skip),
            OpCode::Take => Ok(Instruction::Take),
            OpCode::Length => Ok(Instruction::Length),
            OpCode::Not => Ok(Instruction::Not),
            OpCode::And => Ok(Instruction::And),
            OpCode::Or => Ok(Instruction::Or),
            OpCode::BitwiseAnd => Ok(Instruction::BitwiseAnd),
            OpCode::BitwiseOr => Ok(Instruction::BitwiseOr),
            OpCode::BitwiseXor => Ok(Instruction::BitwiseXor),
            OpCode::BitwiseNeg => Ok(Instruction::BitwiseNeg),
            OpCode::LeftShift => Ok(Instruction::LeftShift),
            OpCode::RightShift => Ok(Instruction::RightShift),
            OpCode::Cons => Ok(Instruction::Cons),
            OpCode::Uncons => Ok(Instruction::Uncons),
            OpCode::First => Ok(Instruction::First),
            OpCode::Second => Ok(Instruction::Second),
            OpCode::Construct => Ok(Instruction::Construct),
            OpCode::Destruct => Ok(Instruction::Destruct),
            OpCode::Leq => Ok(Instruction::Leq),
            OpCode::Lt => Ok(Instruction::Lt),
            OpCode::Geq => Ok(Instruction::Geq),
            OpCode::Gt => Ok(Instruction::Gt),
            OpCode::RefEq => Ok(Instruction::RefEq),
            OpCode::ValEq => Ok(Instruction::ValEq),
            OpCode::RefNeq => Ok(Instruction::RefNeq),
            OpCode::ValNeq => Ok(Instruction::ValNeq),
            OpCode::Call => Ok(Instruction::Call(self.read_offset()?)),
            OpCode::Become => Ok(Instruction::Become(self.read_offset()?)),
            OpCode::Return => Ok(Instruction::Return),
            OpCode::Shift => Ok(Instruction::Shift(self.read_offset()?)),
            OpCode::ShiftBack => Ok(Instruction::ShiftBack(self.read_offset()?)),
            OpCode::Close => Ok(Instruction::Close(self.read_offset()?)),
            OpCode::CloseBack => Ok(Instruction::CloseBack(self.read_offset()?)),
            OpCode::Reset => Ok(Instruction::Reset),
            OpCode::Jump => Ok(Instruction::Jump(self.read_offset()?)),
            OpCode::JumpBack => Ok(Instruction::JumpBack(self.read_offset()?)),
            OpCode::CondJump => Ok(Instruction::CondJump(self.read_offset()?)),
            OpCode::CondJumpBack => Ok(Instruction::CondJumpBack(self.read_offset()?)),
            OpCode::Branch => Ok(Instruction::Branch),
            OpCode::Fizzle => Ok(Instruction::Fizzle),
            OpCode::Exit => Ok(Instruction::Exit),
        }
    }
}

impl Iterator for ProgramReader<'_> {
    type Item = Result<Instruction, InvalidBytecode>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ip == self.program.instructions.len() {
            None
        } else {
            Some(self.read_instruction())
        }
    }
}

pub struct WithIp<'a>(ProgramReader<'a>);

impl<'a> ProgramReader<'a> {
    pub fn with_ip(self) -> WithIp<'a> {
        WithIp(self)
    }
}

impl Iterator for WithIp<'_> {
    type Item = Result<(Offset, Instruction), InvalidBytecode>;

    fn next(&mut self) -> Option<Self::Item> {
        let ip = self.0.ip;
        Some(self.0.next()?.map(|instruction| (ip, instruction)))
    }
}
