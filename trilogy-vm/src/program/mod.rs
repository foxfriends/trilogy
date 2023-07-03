use crate::bytecode::Instruction;
use crate::runtime::Value;
use std::fmt::{self, Display};

mod reader;
mod writer;

use reader::InvalidBytecode;
pub use reader::ProgramReader;

#[derive(Clone, Debug)]
pub struct Program {
    pub(crate) constants: Vec<Value>,
    pub(crate) instructions: Vec<u8>,
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
