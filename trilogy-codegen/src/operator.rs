use crate::{preamble::*, prelude::*};
use trilogy_ir::ir::Builtin;
use trilogy_vm::{Instruction, Struct, Value};

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
        Builtin::Continue => true,
        Builtin::Break => true,
        Builtin::Yield => true,
        Builtin::Resume => true,
        Builtin::Cancel => true,
        Builtin::Pin => true, // Pin is a noop when it appears in an evaluated pattern
        _ => false,
    }
}

pub(crate) fn write_operator(context: &mut Context, builtin: Builtin) {
    match builtin {
        Builtin::Negate => {
            context
                .typecheck(&["number"])
                .instruction(Instruction::Negate);
        }
        Builtin::Not => {
            context
                .typecheck(&["boolean"])
                .instruction(Instruction::Not);
        }
        Builtin::Access => {
            context
                .instruction(Instruction::Swap)
                .typecheck(&["array", "record"])
                .instruction(Instruction::Swap)
                .instruction(Instruction::Access);
        }
        Builtin::And => {
            context
                .typecheck(&["boolean"])
                .instruction(Instruction::Swap)
                .typecheck(&["boolean"])
                .instruction(Instruction::Swap)
                .instruction(Instruction::And);
        }
        Builtin::Or => {
            context
                .typecheck(&["boolean"])
                .instruction(Instruction::Swap)
                .typecheck(&["boolean"])
                .instruction(Instruction::Swap)
                .instruction(Instruction::Or);
        }
        Builtin::Add => {
            context
                .typecheck(&["number"])
                .instruction(Instruction::Swap)
                .typecheck(&["number"])
                .instruction(Instruction::Swap)
                .instruction(Instruction::Add);
        }
        Builtin::Subtract => {
            context
                .typecheck(&["number"])
                .instruction(Instruction::Swap)
                .typecheck(&["number"])
                .instruction(Instruction::Swap)
                .instruction(Instruction::Subtract);
        }
        Builtin::Multiply => {
            context
                .typecheck(&["number"])
                .instruction(Instruction::Swap)
                .typecheck(&["number"])
                .instruction(Instruction::Swap)
                .instruction(Instruction::Multiply);
        }
        Builtin::Divide => {
            context
                .typecheck(&["number"])
                .instruction(Instruction::Swap)
                .typecheck(&["number"])
                .instruction(Instruction::Swap)
                .instruction(Instruction::Divide);
        }
        Builtin::Remainder => {
            context
                .typecheck(&["number"])
                .instruction(Instruction::Swap)
                .typecheck(&["number"])
                .instruction(Instruction::Swap)
                .instruction(Instruction::Remainder);
        }
        Builtin::Power => {
            context
                .typecheck(&["number"])
                .instruction(Instruction::Swap)
                .typecheck(&["number"])
                .instruction(Instruction::Swap)
                .instruction(Instruction::Power);
        }
        Builtin::IntDivide => {
            context
                .typecheck(&["number"])
                .instruction(Instruction::Swap)
                .typecheck(&["number"])
                .instruction(Instruction::Swap)
                .instruction(Instruction::IntDivide);
        }
        Builtin::StructuralEquality => {
            context.instruction(Instruction::ValEq);
        }
        Builtin::StructuralInequality => {
            context.instruction(Instruction::ValNeq);
        }
        Builtin::ReferenceEquality => {
            context.instruction(Instruction::RefEq);
        }
        Builtin::ReferenceInequality => {
            context.instruction(Instruction::RefNeq);
        }
        Builtin::Lt => {
            context.instruction(Instruction::Lt);
        }
        Builtin::Gt => {
            context.instruction(Instruction::Gt);
        }
        Builtin::Leq => {
            context.instruction(Instruction::Leq);
        }
        Builtin::Geq => {
            context.instruction(Instruction::Geq);
        }
        Builtin::BitwiseAnd => {
            context
                .typecheck(&["bits"])
                .instruction(Instruction::Swap)
                .typecheck(&["bits"])
                .instruction(Instruction::Swap)
                .instruction(Instruction::BitwiseAnd);
        }
        Builtin::BitwiseOr => {
            context
                .typecheck(&["bits"])
                .instruction(Instruction::Swap)
                .typecheck(&["bits"])
                .instruction(Instruction::Swap)
                .instruction(Instruction::BitwiseOr);
        }
        Builtin::BitwiseXor => {
            context
                .typecheck(&["bits"])
                .instruction(Instruction::Swap)
                .typecheck(&["bits"])
                .instruction(Instruction::Swap)
                .instruction(Instruction::BitwiseXor);
        }
        Builtin::Invert => {
            context
                .typecheck(&["bits"])
                .instruction(Instruction::BitwiseNeg);
        }
        Builtin::LeftShift => {
            context
                .typecheck(&["number"])
                .instruction(Instruction::Swap)
                .typecheck(&["bits"])
                .instruction(Instruction::Swap)
                .instruction(Instruction::LeftShift);
        }
        Builtin::RightShift => {
            context
                .typecheck(&["number"])
                .instruction(Instruction::Swap)
                .typecheck(&["bits"])
                .instruction(Instruction::Swap)
                .instruction(Instruction::RightShift);
        }
        Builtin::Sequence => {
            context.instruction(Instruction::Pop);
        }
        Builtin::Cons => {
            context.instruction(Instruction::Cons);
        }
        Builtin::Construct => {
            context.instruction(Instruction::Construct);
        }
        Builtin::Glue => {
            context
                .typecheck(&["string"])
                .instruction(Instruction::Swap)
                .typecheck(&["string"])
                .instruction(Instruction::Swap)
                .instruction(Instruction::Glue);
        }
        Builtin::Pipe => {
            context
                .typecheck(&["callable"])
                .instruction(Instruction::Swap);
            apply_function(context);
        }
        Builtin::RPipe => {
            context
                .instruction(Instruction::Swap)
                .typecheck(&["callable"])
                .instruction(Instruction::Swap);
            apply_function(context);
        }
        Builtin::Exit => {
            context.instruction(Instruction::Exit);
        }
        Builtin::Return => {
            context.instruction(Instruction::Return);
        }
        Builtin::Break => {
            let function = context.make_atom("function");
            context
                .instruction(context.scope.kw_break().unwrap())
                .instruction(Instruction::Const(Value::Unit))
                .constant(Struct::new(function, 1))
                .instruction(Instruction::Become(2));
        }
        Builtin::Continue => {
            let function = context.make_atom("function");
            context
                .instruction(context.scope.kw_continue().unwrap())
                .instruction(Instruction::Const(Value::Unit))
                .constant(Struct::new(function, 1))
                .instruction(Instruction::Become(2));
        }
        Builtin::Compose => {
            context
                .write_procedure_reference(RCOMPOSE.to_owned())
                .instruction(Instruction::Swap);
            apply_function(context);
            context.instruction(Instruction::Swap);
            apply_function(context);
        }
        Builtin::RCompose => {
            context
                .write_procedure_reference(COMPOSE.to_owned())
                .instruction(Instruction::Swap);
            apply_function(context);
            context.instruction(Instruction::Swap);
            apply_function(context);
        }
        Builtin::Yield => {
            context
                .write_procedure_reference(YIELD.to_owned())
                .instruction(Instruction::Swap);
            apply_function(context);
        }
        Builtin::Resume => {
            context
                .instruction(context.scope.kw_resume().unwrap())
                .instruction(Instruction::Swap);
            apply_function(context);
        }
        Builtin::Cancel => {
            let function = context.make_atom("function");
            context
                .instruction(context.scope.kw_cancel().unwrap())
                .instruction(Instruction::Swap)
                .constant(Struct::new(function, 1))
                .instruction(Instruction::Become(2));
        }
        Builtin::Pin => {}
        Builtin::ModuleAccess
        | Builtin::Array
        | Builtin::Set
        | Builtin::Record
        | Builtin::Is
        | Builtin::For => {
            panic!("write_operator was called with a builtin that is not an operator")
        }
    };
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
        Builtin::Continue => true,
        Builtin::Break => true,
        Builtin::Return => true,
        Builtin::Resume => true,
        Builtin::Cancel => true,
        _ => false,
    }
}

pub(crate) fn write_operator_reference(context: &mut Context, builtin: Builtin) {
    match builtin {
        Builtin::Negate => context.write_procedure_reference(NEGATE.to_owned()),
        Builtin::Not => context.write_procedure_reference(NOT.to_owned()),
        Builtin::Access => context.write_procedure_reference(ACCESS.to_owned()),
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
        Builtin::Break => context.instruction(context.scope.kw_break().unwrap()),
        Builtin::Continue => context.instruction(context.scope.kw_continue().unwrap()),
        Builtin::Resume => context.instruction(context.scope.kw_resume().unwrap()),
        Builtin::Cancel => context.instruction(context.scope.kw_cancel().unwrap()),
        Builtin::Return => {
            let end = context.labeler.unique_hint("J");
            context.shift(&end);
            unlock_apply(context);
            context.instruction(Instruction::Return).label(end)
        }

        Builtin::ModuleAccess
        | Builtin::Array
        | Builtin::Set
        | Builtin::Record
        | Builtin::Is
        | Builtin::Pin
        | Builtin::For
        | Builtin::Yield
        | Builtin::Sequence
        | Builtin::Construct
        | Builtin::Exit => {
            panic!("write_operator_reference was called with a builtin that is not a referenceable operator")
        }
    };
}
