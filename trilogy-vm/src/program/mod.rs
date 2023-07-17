use crate::bytecode::asm::{AsmContext, AsmError};
use crate::bytecode::Instruction;
use crate::runtime::Value;
use std::collections::HashMap;
use std::fmt::{self, Debug, Display};
use std::str::FromStr;

mod builder;
mod reader;
mod writer;

pub use builder::ProgramBuilder;
use reader::InvalidBytecode;
use reader::ProgramReader;

use self::writer::ProgramWriter;

#[derive(Clone)]
pub struct Program {
    pub(crate) constants: Vec<Value>,
    pub(crate) instructions: Vec<u8>,
    pub(crate) labels: HashMap<String, usize>,
}

impl Debug for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Program {{")?;
        for (i, value) in self.constants.iter().enumerate() {
            writeln!(f, "\t{i}: {value}")?;
        }
        if !self.constants.is_empty() {
            writeln!(f)?;
        }
        let labels_per_line = self.labels.iter().fold(
            HashMap::<usize, Vec<&str>>::new(),
            |mut offsets, (label, offset)| {
                offsets.entry(*offset).or_default().push(label);
                offsets
            },
        );
        let mut width = 0;
        for (i, byte) in self.instructions.iter().enumerate() {
            if let Some(labels) = labels_per_line.get(&i) {
                if width != 0 {
                    writeln!(f)?;
                    width = 0;
                }
                for label in labels {
                    writeln!(f, "{label:?}: ")?;
                }
            }
            if width == 0 {
                write!(f, "\t")?;
            }
            write!(f, "{byte:02X}")?;
            width += 1;
            if width % 25 == 0 {
                writeln!(f)?;
                width = 0;
            } else if width % 5 == 0 {
                write!(f, " ")?;
            }
        }
        if width != 0 {
            writeln!(f)?;
        }
        write!(f, "}}")?;

        Ok(())
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let labels_per_line = self.labels.iter().fold(
            HashMap::<usize, Vec<&str>>::new(),
            |mut offsets, (label, offset)| {
                offsets.entry(*offset).or_default().push(label);
                offsets
            },
        );
        for line in self.into_iter().with_ip() {
            let (ip, instruction) = match line {
                Ok(line) => line,
                Err(..) => {
                    writeln!(f, "Invalid Bytecode.")?;
                    return Ok(());
                }
            };
            for label in labels_per_line.get(&ip).into_iter().flatten() {
                writeln!(f, "{label:?}:")?;
            }
            match &instruction {
                Instruction::Const(Value::Procedure(procedure)) => {
                    if let Some(label) = labels_per_line
                        .get(&procedure.ip())
                        .into_iter()
                        .flatten()
                        .next()
                    {
                        writeln!(f, "\tCONST &{label:?}")?;
                    } else {
                        writeln!(f, "\t{}", instruction)?;
                    }
                }
                Instruction::Jump(offset) => {
                    if let Some(label) = labels_per_line
                        .get(&(ip + offset + 5))
                        .into_iter()
                        .flatten()
                        .next()
                    {
                        writeln!(f, "\tJUMP &{label:?}")?;
                    } else {
                        writeln!(f, "\t{}", instruction)?;
                    }
                }
                Instruction::JumpBack(offset) => {
                    if let Some(label) = labels_per_line
                        .get(&(ip - offset + 5))
                        .into_iter()
                        .flatten()
                        .next()
                    {
                        writeln!(f, "\tRJUMP &{label:?}")?;
                    } else {
                        writeln!(f, "\t{}", instruction)?;
                    }
                }
                Instruction::CondJump(offset) => {
                    if let Some(label) = labels_per_line
                        .get(&(ip + offset + 5))
                        .into_iter()
                        .flatten()
                        .next()
                    {
                        writeln!(f, "\tJUMPF &{label:?}")?;
                    } else {
                        writeln!(f, "\t{}", instruction)?;
                    }
                }
                Instruction::CondJumpBack(offset) => {
                    if let Some(label) = labels_per_line
                        .get(&(ip - offset + 5))
                        .into_iter()
                        .flatten()
                        .next()
                    {
                        writeln!(f, "\tRJUMPF &{label:?}")?;
                    } else {
                        writeln!(f, "\t{}", instruction)?;
                    }
                }
                _ => writeln!(f, "\t{}", instruction)?,
            }
        }
        Ok(())
    }
}

impl<'a> IntoIterator for &'a Program {
    type Item = Result<Instruction, InvalidBytecode>;
    type IntoIter = ProgramReader<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ProgramReader {
            program: self,
            ip: 0,
        }
    }
}

impl FromStr for Program {
    type Err = AsmError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut context = AsmContext::default();
        let mut writer = ProgramWriter::default();
        for instruction in context.parse::<Instruction>(s) {
            writer.write_instruction(instruction?);
        }
        writer.finish(context)
    }
}
