use trilogy_ir::ir::Builtin;
use trilogy_vm::{Instruction, ProgramBuilder};

use crate::labeler::Labeler;

pub(crate) fn is_operator(builtin: Builtin) -> bool {
    #[allow(clippy::match_like_matches_macro)]
    match builtin {
        Builtin::Negate => true,
        Builtin::Not => true,
        Builtin::Access => true,
        Builtin::And => true,
        Builtin::Or => true,
        Builtin::Add => true,
        Builtin::Subtract => true,
        Builtin::Multiply => true,
        Builtin::Divide => true,
        Builtin::Remainder => true,
        Builtin::Power => true,
        Builtin::IntDivide => true,
        Builtin::StructuralEquality => true,
        Builtin::StructuralInequality => true,
        Builtin::ReferenceEquality => true,
        Builtin::ReferenceInequality => true,
        Builtin::Lt => true,
        Builtin::Gt => true,
        Builtin::Leq => true,
        Builtin::Geq => true,
        Builtin::BitwiseAnd => true,
        Builtin::BitwiseOr => true,
        Builtin::BitwiseXor => true,
        Builtin::LeftShift => true,
        Builtin::RightShift => true,
        Builtin::Sequence => true,
        Builtin::Cons => true,
        Builtin::Glue => true,
        Builtin::Pipe => true,
        Builtin::RPipe => true,
        Builtin::Exit => true,
        _ => false,
    }
}

pub(crate) fn write_operator(
    _labeler: &mut Labeler,
    builder: &mut ProgramBuilder,
    builtin: Builtin,
) {
    match builtin {
        Builtin::Negate => builder.write_instruction(Instruction::Negate),
        Builtin::Not => builder.write_instruction(Instruction::Not),
        Builtin::Access => builder.write_instruction(Instruction::Access),
        Builtin::And => builder.write_instruction(Instruction::And),
        Builtin::Or => builder.write_instruction(Instruction::Or),
        Builtin::Add => builder.write_instruction(Instruction::Add),
        Builtin::Subtract => builder.write_instruction(Instruction::Subtract),
        Builtin::Multiply => builder.write_instruction(Instruction::Multiply),
        Builtin::Divide => builder.write_instruction(Instruction::Divide),
        Builtin::Remainder => builder.write_instruction(Instruction::Remainder),
        Builtin::Power => builder.write_instruction(Instruction::Power),
        Builtin::IntDivide => builder.write_instruction(Instruction::IntDivide),
        Builtin::StructuralEquality => builder.write_instruction(Instruction::ValEq),
        Builtin::StructuralInequality => builder.write_instruction(Instruction::ValNeq),
        Builtin::ReferenceEquality => builder.write_instruction(Instruction::RefEq),
        Builtin::ReferenceInequality => builder.write_instruction(Instruction::RefNeq),
        Builtin::Lt => builder.write_instruction(Instruction::Lt),
        Builtin::Gt => builder.write_instruction(Instruction::Gt),
        Builtin::Leq => builder.write_instruction(Instruction::Leq),
        Builtin::Geq => builder.write_instruction(Instruction::Geq),
        Builtin::BitwiseAnd => builder.write_instruction(Instruction::BitwiseAnd),
        Builtin::BitwiseOr => builder.write_instruction(Instruction::BitwiseOr),
        Builtin::BitwiseXor => builder.write_instruction(Instruction::BitwiseXor),
        Builtin::LeftShift => builder.write_instruction(Instruction::LeftShift),
        Builtin::RightShift => builder.write_instruction(Instruction::RightShift),
        Builtin::Sequence => builder.write_instruction(Instruction::Pop),
        Builtin::Cons => builder.write_instruction(Instruction::Cons),
        Builtin::Glue => builder.write_instruction(Instruction::Glue),
        Builtin::Pipe => builder
            .write_instruction(Instruction::Swap)
            .write_instruction(Instruction::Call(1)),
        Builtin::RPipe => builder.write_instruction(Instruction::Call(1)),
        Builtin::Exit => builder.write_instruction(Instruction::Exit),
        _ => panic!("write_operator was called with a builtin that is not an operator"),
    };
}
