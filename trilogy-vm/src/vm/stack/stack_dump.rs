use super::StackCell;
use crate::Value;
use std::fmt::{self, Display};

#[derive(Clone, Debug)]
pub(super) enum DumpCell {
    Unset,
    Frame,
    Set(Value),
}

impl From<StackCell> for DumpCell {
    fn from(value: StackCell) -> Self {
        match value {
            StackCell::Set(value) => Self::Set(value),
            StackCell::Unset => Self::Unset,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StackDump {
    cells: Vec<DumpCell>,
}

impl Display for StackDump {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.cells.is_empty() {
            return write!(f, "--[empty stack]--");
        }
        let mut frame_start = 0;
        for (i, item) in self.cells.iter().enumerate() {
            if i != 0 {
                writeln!(f)?;
            }
            match item {
                DumpCell::Frame => {
                    write!(f, "--[stack frame]--")?;
                    frame_start = i + 1;
                }
                DumpCell::Unset => write!(f, "{}: <empty cell>", i - frame_start)?,
                DumpCell::Set(value) => write!(f, "{}: {value}", i - frame_start)?,
            }
        }
        Ok(())
    }
}

impl FromIterator<DumpCell> for StackDump {
    fn from_iter<T: IntoIterator<Item = DumpCell>>(iter: T) -> Self {
        Self {
            cells: FromIterator::from_iter(iter),
        }
    }
}
