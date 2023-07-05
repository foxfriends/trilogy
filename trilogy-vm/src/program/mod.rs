use crate::bytecode::asm::{AsmContext, AsmError};
use crate::bytecode::Instruction;
use crate::runtime::Value;
use std::collections::HashMap;
use std::fmt::{self, Display};
use std::str::FromStr;

mod reader;
mod writer;

use reader::InvalidBytecode;
pub use reader::ProgramReader;

use self::writer::ProgramWriter;

#[derive(Clone, Debug)]
pub struct Program {
    pub(crate) constants: Vec<Value>,
    pub(crate) instructions: Vec<u8>,
    pub(crate) labels: HashMap<String, usize>,
}

impl Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for instruction in self.into_iter() {
            writeln!(f, "{}", instruction.map_err(|_| fmt::Error)?)?;
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
        let mut writer = ProgramWriter::new();
        for instruction in context.parse::<Instruction>(s) {
            writer.write_instruction(instruction?);
        }
        writer.finish(context)
    }
}
