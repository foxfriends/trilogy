use super::LineAdjuster;
use crate::bytecode::chunk::{Line, Parameter};
use crate::OpCode;

pub(super) fn remove_noops(lines: &mut LineAdjuster) {
    for mut entry in lines {
        if matches!(
            entry.as_line(),
            Line {
                opcode: OpCode::Slide,
                value: Some(Parameter::Offset(0)),
                ..
            }
        ) {
            entry.erase();
        }
    }
}
