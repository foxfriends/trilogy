use crate::bytecode::asm::{AsmContext, AsmError};
use crate::bytecode::OpCode;
use crate::traits::Tags;
use crate::{Instruction, Program, Value};
use std::collections::HashMap;

pub(super) struct ProgramWriter {
    program: Program,
}

impl Default for ProgramWriter {
    fn default() -> Self {
        Self {
            program: Program {
                constants: vec![],
                instructions: vec![],
                labels: HashMap::default(),
            },
        }
    }
}

impl ProgramWriter {
    fn add_constant(&mut self, constant: Value) -> usize {
        self.program
            .constants
            .iter()
            .position(|val| *val == constant)
            .unwrap_or_else(|| {
                let index = self.program.constants.len();
                self.program.constants.push(constant);
                index
            })
    }

    pub fn write_opcode(&mut self, opcode: OpCode) {
        self.program.instructions.push(opcode as u8);
    }

    pub fn write_offset(&mut self, offset: usize) {
        self.program
            .instructions
            .extend((offset as u32).to_be_bytes())
    }

    pub fn finish(mut self, mut context: AsmContext) -> Result<Program, AsmError> {
        for hole_value in context.value_holes() {
            let (hole, value) = hole_value?;
            let offset = u32::from_be_bytes(
                self.program.instructions[hole..hole + 4]
                    .try_into()
                    .unwrap(),
            );
            assert!(matches!(
                self.program.constants[offset as usize],
                Value::Unit,
            ));
            self.program.constants[offset as usize] = value;
        }
        for hole_offset in context.holes() {
            let (hole, offset) = hole_offset?;
            self.program
                .instructions
                .splice(hole..hole + 4, (offset as u32).to_be_bytes());
        }
        self.program.labels = context.labels();
        Ok(self.program)
    }

    pub fn write_instruction(&mut self, instruction: Instruction) {
        self.write_opcode(instruction.tag());
        instruction.write_offset(self);
    }
}

impl Instruction {
    fn write_offset(self, writer: &mut ProgramWriter) {
        let offset = match self {
            Instruction::Const(constant) => {
                let index = writer.add_constant(constant);
                Some(index)
            }
            Instruction::LoadLocal(offset) => Some(offset),
            Instruction::SetLocal(offset) => Some(offset),
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
