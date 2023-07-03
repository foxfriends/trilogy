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
        let value = usize::from_le_bytes(
            self.program.instructions[self.ip..self.ip + 4]
                .try_into()
                .map_err(|_| InvalidBytecode)?,
        );
        self.ip += 4;
        Ok(value)
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
            OpCode::LoadRegister => Ok(Instruction::LoadRegister(self.read_offset()?)),
            OpCode::SetRegister => Ok(Instruction::SetRegister(self.read_offset()?)),
            OpCode::Pop => Ok(Instruction::Pop),
            OpCode::Swap => Ok(Instruction::Swap),
            OpCode::Add => Ok(Instruction::Add),
            OpCode::Copy => Ok(Instruction::Copy),
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
            OpCode::Leq => Ok(Instruction::Leq),
            OpCode::Lt => Ok(Instruction::Lt),
            OpCode::Geq => Ok(Instruction::Geq),
            OpCode::Gt => Ok(Instruction::Gt),
            OpCode::RefEq => Ok(Instruction::RefEq),
            OpCode::ValEq => Ok(Instruction::ValEq),
            OpCode::RefNeq => Ok(Instruction::RefNeq),
            OpCode::ValNeq => Ok(Instruction::ValNeq),
            OpCode::Call => Ok(Instruction::Call(self.read_offset()?)),
            OpCode::Return => Ok(Instruction::Return),
            OpCode::Shift => Ok(Instruction::Shift(self.read_offset()?)),
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
