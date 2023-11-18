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

pub(crate) fn is_unary_operator(builtin: Builtin) -> bool {
    #[allow(clippy::match_like_matches_macro)]
    match builtin {
        Builtin::Negate => true,
        Builtin::Not => true,
        Builtin::Construct => true,
        Builtin::Invert => true,
        Builtin::Exit => true,
        Builtin::Return => true,
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
            context.typecheck("number").instruction(Instruction::Negate);
        }
        Builtin::Not => {
            context.typecheck("boolean").instruction(Instruction::Not);
        }
        Builtin::Access => {
            context
                .instruction(Instruction::Swap)
                .reference(ACCESS)
                .instruction(Instruction::Swap)
                .call_function();
            context.instruction(Instruction::Swap).call_function();
        }
        Builtin::And => {
            context
                .typecheck("boolean")
                .instruction(Instruction::Swap)
                .typecheck("boolean")
                .instruction(Instruction::Swap)
                .instruction(Instruction::And);
        }
        Builtin::Or => {
            context
                .typecheck("boolean")
                .instruction(Instruction::Swap)
                .typecheck("boolean")
                .instruction(Instruction::Swap)
                .instruction(Instruction::Or);
        }
        Builtin::Add => {
            context
                .typecheck("number")
                .instruction(Instruction::Swap)
                .typecheck("number")
                .instruction(Instruction::Swap)
                .instruction(Instruction::Add);
        }
        Builtin::Subtract => {
            context
                .typecheck("number")
                .instruction(Instruction::Swap)
                .typecheck("number")
                .instruction(Instruction::Swap)
                .instruction(Instruction::Subtract);
        }
        Builtin::Multiply => {
            context
                .typecheck("number")
                .instruction(Instruction::Swap)
                .typecheck("number")
                .instruction(Instruction::Swap)
                .instruction(Instruction::Multiply);
        }
        Builtin::Divide => {
            let divided = context.make_label("divided");
            let divzero = context.make_label("divzero");
            context
                .typecheck("number")
                .instruction(Instruction::Swap)
                .typecheck("number")
                .instruction(Instruction::Swap)
                .instruction(Instruction::Copy)
                .constant(0)
                .instruction(Instruction::ValNeq)
                .cond_jump(&divzero)
                .instruction(Instruction::Divide)
                .jump(&divided)
                .label(&divzero)
                .atom("INF");
            write_operator(context, Builtin::Yield);
            context.label(&divided);
        }
        Builtin::Remainder => {
            let remed = context.make_label("remed");
            let remzero = context.make_label("remzero");
            context
                .typecheck("number")
                .instruction(Instruction::Swap)
                .typecheck("number")
                .instruction(Instruction::Swap)
                .instruction(Instruction::Copy)
                .constant(0)
                .instruction(Instruction::ValNeq)
                .cond_jump(&remzero)
                .instruction(Instruction::Remainder)
                .jump(&remed)
                .label(&remzero)
                .atom("INF");
            write_operator(context, Builtin::Yield);
            context.label(&remed);
        }
        Builtin::Power => {
            context
                .typecheck("number")
                .instruction(Instruction::Swap)
                .typecheck("number")
                .instruction(Instruction::Swap)
                .instruction(Instruction::Power);
        }
        Builtin::IntDivide => {
            let intdived = context.make_label("intdived");
            let intdivzero = context.make_label("intdivzero");
            context
                .typecheck("number")
                .instruction(Instruction::Swap)
                .typecheck("number")
                .instruction(Instruction::Swap)
                .instruction(Instruction::Copy)
                .constant(0)
                .instruction(Instruction::ValNeq)
                .cond_jump(&intdivzero)
                .instruction(Instruction::IntDivide)
                .jump(&intdived)
                .label(&intdivzero)
                .atom("INF");
            write_operator(context, Builtin::Yield);
            context.label(&intdived);
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
                .typecheck("bits")
                .instruction(Instruction::Swap)
                .typecheck("bits")
                .instruction(Instruction::Swap)
                .instruction(Instruction::BitwiseAnd);
        }
        Builtin::BitwiseOr => {
            context
                .typecheck("bits")
                .instruction(Instruction::Swap)
                .typecheck("bits")
                .instruction(Instruction::Swap)
                .instruction(Instruction::BitwiseOr);
        }
        Builtin::BitwiseXor => {
            context
                .typecheck("bits")
                .instruction(Instruction::Swap)
                .typecheck("bits")
                .instruction(Instruction::Swap)
                .instruction(Instruction::BitwiseXor);
        }
        Builtin::Invert => {
            context
                .typecheck("bits")
                .instruction(Instruction::BitwiseNeg);
        }
        Builtin::LeftShift => {
            context
                .typecheck("number")
                .instruction(Instruction::Swap)
                .typecheck("bits")
                .instruction(Instruction::Swap)
                .instruction(Instruction::LeftShift);
        }
        Builtin::RightShift => {
            context
                .typecheck("number")
                .instruction(Instruction::Swap)
                .typecheck("bits")
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
                .typecheck("string")
                .instruction(Instruction::Swap)
                .typecheck("string")
                .instruction(Instruction::Swap)
                .instruction(Instruction::Glue);
        }
        Builtin::Pipe => {
            context
                .typecheck("callable")
                .instruction(Instruction::Swap)
                .call_function();
        }
        Builtin::RPipe => {
            context
                .instruction(Instruction::Swap)
                .typecheck("callable")
                .instruction(Instruction::Swap)
                .call_function();
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
                .reference(RCOMPOSE.to_owned())
                .instruction(Instruction::Swap)
                .call_function();
            context.instruction(Instruction::Swap).call_function();
        }
        Builtin::RCompose => {
            context
                .reference(COMPOSE.to_owned())
                .instruction(Instruction::Swap)
                .call_function();
            context.instruction(Instruction::Swap).call_function();
        }
        Builtin::Yield => {
            context.r#yield();
        }
        Builtin::Resume => {
            context
                .instruction(context.scope.kw_resume().unwrap())
                .instruction(Instruction::Swap)
                .call_function();
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
        Builtin::Not => context.reference(NOT),
        Builtin::Access => context.reference(ACCESS),
        Builtin::And => context.reference(AND),
        Builtin::Or => context.reference(OR),
        Builtin::Add => context.reference(ADD),
        Builtin::Subtract => context.reference(SUB),
        Builtin::Multiply => context.reference(MUL),
        Builtin::Divide => context.reference(DIV),
        Builtin::Remainder => context.reference(REM),
        Builtin::Power => context.reference(POW),
        Builtin::IntDivide => context.reference(INTDIV),
        Builtin::StructuralEquality => context.reference(VALEQ),
        Builtin::StructuralInequality => context.reference(VALNEQ),
        Builtin::ReferenceEquality => context.reference(REFEQ),
        Builtin::ReferenceInequality => context.reference(REFNEQ),
        Builtin::Lt => context.reference(LT),
        Builtin::Gt => context.reference(GT),
        Builtin::Leq => context.reference(LEQ),
        Builtin::Geq => context.reference(GEQ),
        Builtin::BitwiseAnd => context.reference(BITAND),
        Builtin::BitwiseOr => context.reference(BITOR),
        Builtin::BitwiseXor => context.reference(BITXOR),
        Builtin::Invert => context.reference(BITNEG),
        Builtin::LeftShift => context.reference(LSHIFT),
        Builtin::RightShift => context.reference(RSHIFT),
        Builtin::Cons => context.reference(CONS),
        Builtin::Glue => context.reference(GLUE),
        Builtin::Pipe => context.reference(PIPE),
        Builtin::RPipe => context.reference(RPIPE),
        Builtin::Compose => context.reference(COMPOSE),
        Builtin::RCompose => context.reference(RCOMPOSE),
        Builtin::Break => context.instruction(context.scope.kw_break().unwrap()),
        Builtin::Continue => context.instruction(context.scope.kw_continue().unwrap()),
        Builtin::Resume => context.instruction(context.scope.kw_resume().unwrap()),
        Builtin::Cancel => context.instruction(context.scope.kw_cancel().unwrap()),
        Builtin::Return => {
            context.continuation_fn(|context| {
                context.instruction(Instruction::Return);
            })
        }

        Builtin::Negate
        | Builtin::ModuleAccess
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
