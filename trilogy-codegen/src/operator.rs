use crate::{preamble::*, prelude::*};
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
        Builtin::Construct => true,
        Builtin::Lt => true,
        Builtin::Gt => true,
        Builtin::Leq => true,
        Builtin::Geq => true,
        Builtin::Invert => true,
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
        Builtin::Compose => true,
        Builtin::RCompose => true,
        _ => false,
    }
}

pub(crate) fn is_referenceable_operator(builtin: Builtin) -> bool {
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
        Builtin::Invert => true,
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
        Builtin::Compose => true,
        Builtin::RCompose => true,
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
        Builtin::Invert => context.write_instruction(Instruction::BitwiseNeg),
        Builtin::LeftShift => context.write_instruction(Instruction::LeftShift),
        Builtin::RightShift => context.write_instruction(Instruction::RightShift),
        Builtin::Sequence => context.write_instruction(Instruction::Pop),
        Builtin::Cons => context.write_instruction(Instruction::Cons),
        Builtin::Construct => context.write_instruction(Instruction::Construct),
        Builtin::Glue => context.write_instruction(Instruction::Glue),
        Builtin::Pipe => context
            .write_instruction(Instruction::Swap)
            .write_instruction(Instruction::Call(1)),
        Builtin::RPipe => context.write_instruction(Instruction::Call(1)),
        Builtin::Exit => context.write_instruction(Instruction::Exit),
        Builtin::Return => context.write_instruction(context.scope.kw_return()),
        Builtin::Compose => context
            .write_procedure_reference(RCOMPOSE.to_owned())
            .write_instruction(Instruction::Swap)
            .write_instruction(Instruction::Call(1))
            .write_instruction(Instruction::Swap)
            .write_instruction(Instruction::Call(1)),
        Builtin::RCompose => context
            .write_procedure_reference(COMPOSE.to_owned())
            .write_instruction(Instruction::Swap)
            .write_instruction(Instruction::Call(1))
            .write_instruction(Instruction::Swap)
            .write_instruction(Instruction::Call(1)),
        Builtin::ModuleAccess
        | Builtin::Array
        | Builtin::Set
        | Builtin::Record
        | Builtin::Is
        | Builtin::Pin
        | Builtin::For
        | Builtin::Yield
        | Builtin::Resume
        | Builtin::Cancel
        | Builtin::Break
        | Builtin::Continue => {
            panic!("write_operator was called with a builtin that is not an operator")
        }
    };
}

pub(crate) fn write_operator_reference(context: &mut Context, builtin: Builtin) {
    match builtin {
        Builtin::Negate => context.write_procedure_reference(NEGATE.to_owned()),
        Builtin::Not => context.write_procedure_reference(NOT.to_owned()),
        Builtin::Access => todo!(),
        Builtin::And => context.write_procedure_reference(AND.to_owned()),
        Builtin::Or => context.write_procedure_reference(OR.to_owned()),
        Builtin::Add => context.write_procedure_reference(ADD.to_owned()),
        Builtin::Subtract => context.write_procedure_reference(SUB.to_owned()),
        Builtin::Multiply => context.write_procedure_reference(MUL.to_owned()),
        Builtin::Divide => context.write_procedure_reference(DIV.to_owned()),
        Builtin::Remainder => context.write_procedure_reference(REM.to_owned()),
        Builtin::Power => context.write_procedure_reference(POW.to_owned()),
        Builtin::IntDivide => context.write_procedure_reference(INTDIV.to_owned()),
        Builtin::StructuralEquality => context.write_procedure_reference(VALEQ.to_owned()),
        Builtin::StructuralInequality => context.write_procedure_reference(VALNEQ.to_owned()),
        Builtin::ReferenceEquality => context.write_procedure_reference(REFEQ.to_owned()),
        Builtin::ReferenceInequality => context.write_procedure_reference(REFNEQ.to_owned()),
        Builtin::Lt => context.write_procedure_reference(LT.to_owned()),
        Builtin::Gt => context.write_procedure_reference(GT.to_owned()),
        Builtin::Leq => context.write_procedure_reference(LEQ.to_owned()),
        Builtin::Geq => context.write_procedure_reference(GEQ.to_owned()),
        Builtin::BitwiseAnd => context.write_procedure_reference(BITAND.to_owned()),
        Builtin::BitwiseOr => context.write_procedure_reference(BITOR.to_owned()),
        Builtin::BitwiseXor => context.write_procedure_reference(BITXOR.to_owned()),
        Builtin::Invert => context.write_procedure_reference(BITNEG.to_owned()),
        Builtin::LeftShift => context.write_procedure_reference(LSHIFT.to_owned()),
        Builtin::RightShift => context.write_procedure_reference(RSHIFT.to_owned()),
        Builtin::Cons => context.write_procedure_reference(CONS.to_owned()),
        Builtin::Glue => context.write_procedure_reference(GLUE.to_owned()),
        Builtin::Pipe => context.write_procedure_reference(PIPE.to_owned()),
        Builtin::RPipe => context.write_procedure_reference(RPIPE.to_owned()),
        Builtin::Compose => context.write_procedure_reference(COMPOSE.to_owned()),
        Builtin::RCompose => context.write_procedure_reference(RCOMPOSE.to_owned()),

        Builtin::ModuleAccess
        | Builtin::Array
        | Builtin::Set
        | Builtin::Record
        | Builtin::Is
        | Builtin::Pin
        | Builtin::For
        | Builtin::Yield
        | Builtin::Resume
        | Builtin::Cancel
        | Builtin::Break
        | Builtin::Continue
        | Builtin::Sequence
        | Builtin::Construct
        | Builtin::Return
        | Builtin::Exit => {
            panic!("write_operator_reference was called with a builtin that is not a referenceable operator")
        }
    };
}
