use super::{InternalValue, Stack};
use crate::vm::execution::Cont;
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

impl Stack {
    pub(in super::super) fn trace(&self, program: &ProgramReader, ip: Offset) -> StackTrace {
        let mut trace = StackTrace::default();
        let annotations = program.annotations(ip);
        trace.frames.push(StackTraceEntry {
            annotations: annotations
                .into_iter()
                .filter_map(|annotation| annotation.note.into_source())
                .collect(),
        });

        trace
            .frames
            .extend(self.cactus.clone().into_iter().filter_map(|entry| {
                match entry {
                    InternalValue::Return { cont, .. } => match cont {
                        Cont::Callback(..) => Some(StackTraceEntry {
                            annotations: vec![],
                        }),
                        Cont::Offset(ip) => Some(StackTraceEntry {
                            annotations: program
                                .annotations(ip)
                                .into_iter()
                                .filter_map(|annotation| annotation.note.into_source())
                                .collect(),
                        }),
                    },
                    _ => None,
                }
            }));

        trace.frames.reverse();
        trace
    }
}
