use crate::bytecode::asm::{Asm, AsmContext, AsmError};
use crate::bytecode::OpCode;
use crate::{Instruction, Program, Value};
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct Error {
    pub line: usize,
    pub error: AsmError,
}

impl FromStr for Program {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut context = AsmContext::default();
        let mut program = Program {
            constants: vec![],
            instructions: vec![],
        };
        let mut writer = ProgramWriter {
            program: &mut program,
        };

        let instructions = s
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .filter(|line| !line.starts_with('#'))
            .map(|line| Instruction::parse_asm(line, &mut context))
            .enumerate()
            .map(|(line, result)| result.map_err(|error| Error { line, error }));

        for instruction in instructions {
            match instruction? {
                Instruction::Const(constant) => {
                    let index = writer.add_constant(constant);
                    writer.write_opcode(OpCode::Const);
                    writer.write_offset(index);
                }
                Instruction::Load => writer.write_opcode(OpCode::Load),
                Instruction::Set => writer.write_opcode(OpCode::Set),
                Instruction::Alloc => writer.write_opcode(OpCode::Alloc),
                Instruction::Free => writer.write_opcode(OpCode::Free),
                Instruction::LoadRegister(offset) => {
                    writer.write_opcode(OpCode::LoadRegister);
                    writer.write_offset(offset);
                }
                Instruction::SetRegister(offset) => {
                    writer.write_opcode(OpCode::SetRegister);
                    writer.write_offset(offset);
                }
                Instruction::Copy => writer.write_opcode(OpCode::Copy),
                Instruction::Pop => writer.write_opcode(OpCode::Pop),
                Instruction::Swap => writer.write_opcode(OpCode::Swap),
                Instruction::Add => writer.write_opcode(OpCode::Add),
                Instruction::Subtract => writer.write_opcode(OpCode::Subtract),
                Instruction::Multiply => writer.write_opcode(OpCode::Multiply),
                Instruction::Divide => writer.write_opcode(OpCode::Divide),
                Instruction::Remainder => writer.write_opcode(OpCode::Remainder),
                Instruction::IntDivide => writer.write_opcode(OpCode::IntDivide),
                Instruction::Power => writer.write_opcode(OpCode::Power),
                Instruction::Negate => writer.write_opcode(OpCode::Negate),
                Instruction::Glue => writer.write_opcode(OpCode::Glue),
                Instruction::Access => writer.write_opcode(OpCode::Access),
                Instruction::Assign => writer.write_opcode(OpCode::Assign),
                Instruction::Not => writer.write_opcode(OpCode::Not),
                Instruction::And => writer.write_opcode(OpCode::And),
                Instruction::Or => writer.write_opcode(OpCode::Or),
                Instruction::BitwiseAnd => writer.write_opcode(OpCode::BitwiseAnd),
                Instruction::BitwiseOr => writer.write_opcode(OpCode::BitwiseOr),
                Instruction::BitwiseXor => writer.write_opcode(OpCode::BitwiseXor),
                Instruction::BitwiseNeg => writer.write_opcode(OpCode::BitwiseNeg),
                Instruction::LeftShift => writer.write_opcode(OpCode::LeftShift),
                Instruction::RightShift => writer.write_opcode(OpCode::RightShift),
                Instruction::Cons => writer.write_opcode(OpCode::Cons),
                Instruction::Leq => writer.write_opcode(OpCode::Leq),
                Instruction::Lt => writer.write_opcode(OpCode::Lt),
                Instruction::Geq => writer.write_opcode(OpCode::Geq),
                Instruction::Gt => writer.write_opcode(OpCode::Gt),
                Instruction::RefEq => writer.write_opcode(OpCode::RefEq),
                Instruction::ValEq => writer.write_opcode(OpCode::ValEq),
                Instruction::RefNeq => writer.write_opcode(OpCode::RefNeq),
                Instruction::ValNeq => writer.write_opcode(OpCode::ValNeq),
                Instruction::Call(offset) => {
                    writer.write_opcode(OpCode::Call);
                    writer.write_offset(offset);
                }
                Instruction::Return => writer.write_opcode(OpCode::Return),
                Instruction::Shift(offset) => {
                    writer.write_opcode(OpCode::Shift);
                    writer.write_offset(offset);
                }
                Instruction::Reset => writer.write_opcode(OpCode::Reset),
                Instruction::Jump(offset) => {
                    writer.write_opcode(OpCode::Jump);
                    writer.write_offset(offset);
                }
                Instruction::JumpBack(offset) => {
                    writer.write_opcode(OpCode::JumpBack);
                    writer.write_offset(offset);
                }
                Instruction::CondJump(offset) => {
                    writer.write_opcode(OpCode::CondJump);
                    writer.write_offset(offset);
                }
                Instruction::CondJumpBack(offset) => {
                    writer.write_opcode(OpCode::CondJumpBack);
                    writer.write_offset(offset);
                }
                Instruction::Branch => writer.write_opcode(OpCode::Branch),
                Instruction::Fizzle => writer.write_opcode(OpCode::Fizzle),
                Instruction::Exit => writer.write_opcode(OpCode::Exit),
            }
        }

        Ok(program)
    }
}

struct ProgramWriter<'a> {
    program: &'a mut Program,
}

impl ProgramWriter<'_> {
    fn add_constant(&mut self, constant: Value) -> usize {
        let index = self.program.constants.len();
        self.program.constants.push(constant);
        index
    }

    fn write_opcode(&mut self, opcode: OpCode) {
        self.program.instructions.push(opcode as u8);
    }

    fn write_offset(&mut self, offset: usize) {
        self.program
            .instructions
            .extend((offset as u32).to_be_bytes())
    }
}
