use crate::Context;
use trilogy_ir::ir::Builtin;
use trilogy_vm::Instruction;

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
        Builtin::Return => true,
        _ => false,
    }
}

pub(crate) fn write_operator(context: &mut Context, builtin: Builtin) {
    match builtin {
        Builtin::Negate => context.write_instruction(Instruction::Negate),
        Builtin::Not => context.write_instruction(Instruction::Not),
        Builtin::Access => context.write_instruction(Instruction::Access),
        Builtin::And => context.write_instruction(Instruction::And),
        Builtin::Or => context.write_instruction(Instruction::Or),
        Builtin::Add => context.write_instruction(Instruction::Add),
        Builtin::Subtract => context.write_instruction(Instruction::Subtract),
        Builtin::Multiply => context.write_instruction(Instruction::Multiply),
        Builtin::Divide => context.write_instruction(Instruction::Divide),
        Builtin::Remainder => context.write_instruction(Instruction::Remainder),
        Builtin::Power => context.write_instruction(Instruction::Power),
        Builtin::IntDivide => context.write_instruction(Instruction::IntDivide),
        Builtin::StructuralEquality => context.write_instruction(Instruction::ValEq),
        Builtin::StructuralInequality => context.write_instruction(Instruction::ValNeq),
        Builtin::ReferenceEquality => context.write_instruction(Instruction::RefEq),
        Builtin::ReferenceInequality => context.write_instruction(Instruction::RefNeq),
        Builtin::Lt => context.write_instruction(Instruction::Lt),
        Builtin::Gt => context.write_instruction(Instruction::Gt),
        Builtin::Leq => context.write_instruction(Instruction::Leq),
        Builtin::Geq => context.write_instruction(Instruction::Geq),
        Builtin::BitwiseAnd => context.write_instruction(Instruction::BitwiseAnd),
        Builtin::BitwiseOr => context.write_instruction(Instruction::BitwiseOr),
        Builtin::BitwiseXor => context.write_instruction(Instruction::BitwiseXor),
        Builtin::LeftShift => context.write_instruction(Instruction::LeftShift),
        Builtin::RightShift => context.write_instruction(Instruction::RightShift),
        Builtin::Sequence => context.write_instruction(Instruction::Pop),
        Builtin::Cons => context.write_instruction(Instruction::Cons),
        Builtin::Glue => context.write_instruction(Instruction::Glue),
        Builtin::Pipe => context
            .write_instruction(Instruction::Swap)
            .write_instruction(Instruction::Call(1)),
        Builtin::RPipe => context.write_instruction(Instruction::Call(1)),
        Builtin::Exit => context.write_instruction(Instruction::Exit),
        Builtin::Return => context.write_instruction(Instruction::Return),
        _ => panic!("write_operator was called with a builtin that is not an operator"),
    };
}
