use crate::bytecode::asm::{Asm, AsmContext, AsmError};
use crate::bytecode::OpCode;
use crate::traits::Tags;
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
            let instruction = instruction?;
            writer.write_opcode(instruction.tag());
            instruction.write_offset(&mut writer);
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

impl Instruction {
    fn write_offset(self, writer: &mut ProgramWriter) {
        let offset = match self {
            Instruction::Const(constant) => {
                let index = writer.add_constant(constant);
                Some(index)
            }
            Instruction::LoadRegister(offset) => Some(offset),
            Instruction::SetRegister(offset) => Some(offset),
            Instruction::Call(offset) => Some(offset),
            Instruction::Shift(offset) => Some(offset),
            Instruction::Jump(offset) => Some(offset),
            Instruction::JumpBack(offset) => Some(offset),
            Instruction::CondJump(offset) => Some(offset),
            Instruction::CondJumpBack(offset) => Some(offset),
            _ => None,
        };
        if let Some(offset) = offset {
            writer.write_offset(offset);
        }
    }
}
