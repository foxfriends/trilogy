use super::{Cont, Stack};
use crate::vm::program_reader::ProgramReader;
use crate::{Location, Offset};

#[derive(Clone, Debug)]
pub struct StackTraceEntry {
    pub ip: Option<Offset>,
    pub source_annotations: Vec<(String, Location)>,
    pub notes: Vec<String>,
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
            ip: Some(ip),
            source_annotations: annotations
                .iter()
                .cloned()
                .chain(
                    self.frames
                        .last()
                        .and_then(|frame| frame.here)
                        .into_iter()
                        .flat_map(|ip| program.annotations(ip)),
                )
                .filter_map(|annotation| annotation.note.into_source())
                .collect(),
            notes: annotations
                .iter()
                .cloned()
                .chain(
                    self.frames
                        .last()
                        .and_then(|frame| frame.here)
                        .into_iter()
                        .flat_map(|ip| program.annotations(ip)),
                )
                .filter_map(|annotation| annotation.note.into_note())
                .collect(),
        });
        let stack_frames = self.frames().rev().map(|entry| match entry.cont {
            Cont::Callback(..) => StackTraceEntry {
                ip: None,
                source_annotations: vec![],
                notes: vec![],
            },
            Cont::Offset(ip) => StackTraceEntry {
                ip: Some(ip),
                source_annotations: program
                    .annotations(ip)
                    .iter()
                    .cloned()
                    .filter_map(|annotation| annotation.note.into_source())
                    .collect(),
                notes: program
                    .annotations(ip)
                    .iter()
                    .cloned()
                    .filter_map(|annotation| annotation.note.into_note())
                    .collect(),
            },
        });
        trace.frames.extend(stack_frames);
        trace
    }
}
