use super::{Cont, Stack};
use crate::vm::program_reader::ProgramReader;
use crate::{Location, Offset};

#[derive(Clone, Debug)]
pub struct StackTraceEntry {
    pub annotations: Vec<(String, Location)>,
}

#[derive(Clone, Debug, Default)]
pub struct StackTrace {
    pub frames: Vec<StackTraceEntry>,
}

impl Stack<'_> {
    pub(in super::super) fn trace(&self, program: &ProgramReader, ip: Offset) -> StackTrace {
        let mut trace = StackTrace::default();
        let annotations = program.annotations(ip);
        trace.frames.push(StackTraceEntry {
            annotations: annotations
                .into_iter()
                .filter_map(|annotation| annotation.note.into_source())
                .collect(),
        });

        trace.frames.extend(self.frames().filter_map(|entry| {
            Some({
                match entry.cont {
                    Cont::Callback(..) => StackTraceEntry {
                        annotations: vec![],
                    },
                    Cont::Offset(ip) => StackTraceEntry {
                        annotations: program
                            .annotations(ip)
                            .into_iter()
                            .filter_map(|annotation| annotation.note.into_source())
                            .collect(),
                    },
                }
            })
        }));

        trace.frames.reverse();
        trace
    }
}
