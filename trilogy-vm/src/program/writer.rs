use crate::bytecode::asm::{AsmContext, AsmError};
use crate::bytecode::OpCode;
use crate::traits::Tags;
use crate::{Instruction, Program, Value};
use std::str::FromStr;

impl FromStr for Program {
    type Err = AsmError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut context = AsmContext::default();
        let mut writer = ProgramWriter::new();
        for instruction in context.parse::<Instruction>(s) {
            let instruction = instruction?;
            writer.write_opcode(instruction.tag());
            instruction.write_offset(&mut writer);
        }
        writer.finish(context)
    }
}

struct ProgramWriter {
    program: Program,
}

impl ProgramWriter {
    fn new() -> Self {
        Self {
            program: Program {
                constants: vec![],
                instructions: vec![],
            },
        }
    }

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

    fn finish(mut self, context: AsmContext) -> Result<Program, AsmError> {
        for hole_offset in context.holes() {
            let (hole, offset) = hole_offset?;
            self.program
                .instructions
                .splice(hole..hole + 4, (offset as u32).to_be_bytes());
        }
        Ok(self.program)
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
