use crate::{Offset, OpCode};
use std::fmt::{self, Display};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Duration;

/// Similar to `std::time::Duration`, but now atomic.
///
/// Does not provide the best or any guarantees, but is suitable for the purpose
/// of an atomic duration total.
#[derive(Default, Debug)]
pub struct AtomicDuration(AtomicU64, AtomicU32);

impl AtomicDuration {
    pub fn fetch_add(&self, duration: Duration, ordering: Ordering) -> Duration {
        let secs = duration.as_secs();
        let sec_nanos = secs as u128 * 1_000_000_000;
        let nanos = (duration.as_nanos() - sec_nanos) as u32;
        let secs = self.0.fetch_add(secs, ordering);
        let nanos = self.1.fetch_add(nanos, ordering);
        Duration::new(secs, nanos)
    }

    pub fn load(&self, ordering: Ordering) -> Duration {
        let secs = self.0.load(ordering);
        let nanos = self.1.load(ordering);
        Duration::new(secs, nanos)
    }
}

#[derive(Debug)]
pub struct Stats {
    pub instructions_executed: [AtomicU64; OpCode::Debug as usize],
    pub instruction_timing: [AtomicDuration; OpCode::Debug as usize],
    pub instruction_read_duration: AtomicDuration,
    pub native_duration: AtomicDuration,
    pub branch_hits: AtomicU64,
    pub branch_misses: AtomicU64,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            instructions_executed: std::array::from_fn(|_| AtomicU64::default()),
            instruction_timing: std::array::from_fn(|_| AtomicDuration::default()),
            instruction_read_duration: AtomicDuration::default(),
            native_duration: AtomicDuration::default(),
            branch_hits: AtomicU64::default(),
            branch_misses: AtomicU64::default(),
        }
    }
}

impl Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let total_instructions = self
            .instructions_executed
            .iter()
            .fold(0, |a, b| a + b.load(Ordering::Relaxed));
        let total_duration = self
            .instruction_timing
            .iter()
            .fold(Duration::default(), |a, b| a + b.load(Ordering::Relaxed));
        writeln!(f, "--- Basic Statistics ---")?;
        writeln!(
            f,
            "Read instruction duration: {:?} (avg: {:?})",
            self.instruction_read_duration.load(Ordering::Relaxed),
            self.instruction_read_duration.load(Ordering::Relaxed) / total_instructions as u32,
        )?;
        writeln!(
            f,
            "     Native call duration: {:?}",
            self.native_duration.load(Ordering::Relaxed)
        )?;
        writeln!(
            f,
            "           Total duration: {:?}",
            self.instruction_read_duration.load(Ordering::Relaxed) + total_duration,
        )?;

        writeln!(
            f,
            "JUMPF accuracy: {:>8} hits",
            self.branch_hits.load(Ordering::Relaxed)
        )?;
        writeln!(
            f,
            "                {:>8} miss",
            self.branch_misses.load(Ordering::Relaxed)
        )?;

        let max_times = self
            .instructions_executed
            .iter()
            .fold(0, |a, b| u64::max(a, b.load(Ordering::Relaxed)));
        writeln!(f, "--- Instructions Executed ---")?;
        for (opcode, times) in self.instructions_executed.iter().enumerate() {
            let times = times.load(Ordering::Relaxed);
            if times == 0 {
                continue;
            }
            writeln!(
                f,
                "{:>10} {:>16} {}",
                OpCode::try_from(opcode as Offset).unwrap(),
                times,
                "#".repeat((times * 50 / max_times) as usize)
            )?;
        }
        writeln!(f, "Total: {}", total_instructions,)?;

        writeln!(f, "--- Instruction Timing ---")?;
        let max_duration = self
            .instruction_timing
            .iter()
            .enumerate()
            .filter(|(op, _)| *op != OpCode::Chunk as usize)
            .fold(Duration::default(), |a, (_, b)| {
                Duration::max(a, b.load(Ordering::Relaxed))
            });
        for (opindex, duration) in self.instruction_timing.iter().enumerate() {
            let opcode = OpCode::try_from(opindex as Offset).unwrap();
            let duration = duration.load(Ordering::Relaxed);
            if duration.is_zero() {
                continue;
            }
            writeln!(
                f,
                "{:>10} {:>16?} {}",
                opcode,
                duration,
                if opcode == OpCode::Chunk {
                    "[unmeasured]".to_owned()
                } else {
                    let duration = (duration.as_nanos() * 50 / max_duration.as_nanos()) as usize;
                    "#".repeat(duration)
                }
            )?;
        }
        writeln!(f, "Total: {:?}", total_duration,)?;

        writeln!(f, "--- Average Duration ---")?;
        let max_average = self
            .instruction_timing
            .iter()
            .enumerate()
            .filter(|(op, _)| *op != OpCode::Chunk as usize)
            .fold(Duration::default(), |a, (opcode, b)| {
                let times = self.instructions_executed[opcode].load(Ordering::Relaxed) as u32;
                if times == 0 {
                    return a;
                }
                Duration::max(a, b.load(Ordering::Relaxed) / times)
            });
        for (opindex, duration) in self.instruction_timing.iter().enumerate() {
            let opcode = OpCode::try_from(opindex as Offset).unwrap();
            let duration = duration.load(Ordering::Relaxed);
            let times = self.instructions_executed[opindex].load(Ordering::Relaxed);
            if duration.is_zero() || times == 0 {
                continue;
            }
            let average = duration / times as u32;
            writeln!(
                f,
                "{:>10} {:>16?} {}",
                opcode,
                average,
                if opcode == OpCode::Chunk {
                    "[unmeasured]".to_owned()
                } else {
                    let percentage = (average.as_nanos() * 50 / max_average.as_nanos()) as usize;
                    "#".repeat(percentage)
                }
            )?;
        }
        Ok(())
    }
}
