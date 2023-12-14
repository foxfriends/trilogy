use crate::OpCode;
use std::collections::BTreeMap;
use std::fmt::{self, Display};
use std::time::Duration;

#[derive(Clone, Default, Debug)]
pub struct Stats {
    pub instructions_executed: BTreeMap<OpCode, usize>,
    pub instruction_timing: BTreeMap<OpCode, Duration>,
}

impl Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let max_times = self
            .instructions_executed
            .values()
            .fold(0, |a, b| usize::max(a, *b));
        writeln!(f, "--- Instructions Executed ---")?;
        for (opcode, times) in &self.instructions_executed {
            writeln!(
                f,
                "{:>10} {:>16} {}",
                opcode,
                times,
                "=".repeat(times * 50 / max_times)
            )?;
        }
        writeln!(
            f,
            "Total: {:?}",
            self.instructions_executed.values().fold(0, |a, b| a + *b)
        )?;

        writeln!(f, "--- Instruction Timing ---")?;
        let max_duration = self
            .instruction_timing
            .values()
            .fold(Duration::default(), |a, b| Duration::max(a, *b));
        for (opcode, duration) in &self.instruction_timing {
            writeln!(
                f,
                "{:>10} {:>16?} {}",
                opcode,
                duration,
                "=".repeat((duration.as_nanos() * 50 / max_duration.as_nanos()) as usize)
            )?;
        }
        writeln!(
            f,
            "Total: {:?}",
            self.instruction_timing
                .values()
                .fold(Duration::default(), |a, b| a + *b)
        )?;

        writeln!(f, "--- Average Duration ---")?;
        let max_average = self
            .instruction_timing
            .iter()
            .filter(|(op, _)| **op != OpCode::Chunk)
            .fold(Duration::default(), |a, (opcode, b)| {
                Duration::max(
                    a,
                    *b / (*self.instructions_executed.get(opcode).unwrap()) as u32,
                )
            });
        for (opcode, duration) in &self.instruction_timing {
            let times = *self.instructions_executed.get(opcode).unwrap() as u32;
            let average = *duration / times as u32;
            writeln!(
                f,
                "{:>10} {:>16?} {}",
                opcode,
                average,
                if *opcode == OpCode::Chunk {
                    "[unmeasured]".to_owned()
                } else {
                    let percentage = (average.as_nanos() * 50 / max_average.as_nanos()) as usize;
                    "=".repeat(percentage)
                }
            )?;
        }
        Ok(())
    }
}
