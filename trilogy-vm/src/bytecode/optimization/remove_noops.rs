use crate::bytecode::chunk::{Line, Parameter};
use crate::OpCode;

pub(super) fn remove_noops(mut lines: Vec<Line>) -> Vec<Line> {
    let mut i = 0;
    while i < lines.len() {
        match &lines[i] {
            Line {
                opcode: OpCode::Slide,
                value: Some(Parameter::Offset(0)),
                ..
            } => {
                let Line { labels, .. } = lines.remove(i);
                lines[i - 1].labels.extend(labels);
            }
            _ => {
                i += 1;
            }
        }
    }
    lines
}
