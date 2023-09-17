use crate::Value;
use std::collections::HashMap;
use std::fmt::Debug;

mod builder;

pub use builder::ChunkBuilder;

/// A chunk of independently compiled source code for this VM.
#[derive(Clone)]
pub struct Chunk {
    labels: HashMap<String, u32>,
    pub(crate) constants: Vec<Value>,
    pub(crate) bytes: Vec<u8>,
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Constants:")?;
        for (i, constant) in self.constants.iter().enumerate() {
            writeln!(f, "{i}: {constant:?}")?;
        }

        let mut needs_break = false;
        for (i, byte) in self.bytes.iter().enumerate() {
            for (label, _) in self
                .labels
                .iter()
                .filter(|(_, offset)| **offset == i as u32)
            {
                if needs_break {
                    writeln!(f)?;
                    needs_break = false;
                }
                writeln!(f, "{label}:")?;
            }
            write!(f, "{byte:02x}")?;
            needs_break = true;
        }
        writeln!(f)
    }
}
