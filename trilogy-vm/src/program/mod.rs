use crate::bytecode::asm::{AsmContext, AsmError};
use crate::bytecode::Instruction;
use crate::runtime::Value;
use std::collections::HashMap;
use std::fmt::{self, Display};
use std::str::FromStr;

mod builder;
mod reader;
mod writer;

pub use builder::ProgramBuilder;
use reader::InvalidBytecode;
use reader::ProgramReader;

use self::writer::ProgramWriter;

#[derive(Clone, Debug)]
pub struct Program {
    pub(crate) constants: Vec<Value>,
    pub(crate) instructions: Vec<u8>,
    pub(crate) labels: HashMap<String, usize>,
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
        let mut offset = 0;
        let mut instructions = self.into_iter();
        loop {
            for label in labels_per_line.get(&offset).into_iter().flatten() {
                writeln!(f, "{label:?}:")?;
            }
            let Some(instruction) = instructions.next() else { break };
            let instruction = instruction.map_err(|_| fmt::Error)?;
            writeln!(f, "\t{}", instruction)?;
            offset += instruction.size();
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
