use crate::prelude::*;
use trilogy_ir::ir::{self, Builtin, Expression};
use trilogy_vm::{Instruction, Value};

#[inline(always)]
pub(crate) fn write_pattern_match(context: &mut Context, expr: &Expression, on_fail: &str) {
    write_pattern(context, &expr.value, on_fail);
}

/// Pattern matches the top of the stack with an expression.
/// * On success, the binding references found in the pattern have been set to the appropriate values.
/// * On failure, the provided label is jumped to.
/// * In either case, the original value is consumed.
pub(crate) fn write_pattern(context: &mut Context, value: &ir::Value, on_fail: &str) {
    match &value {
        ir::Value::Mapping(..) => todo!(),
        ir::Value::Number(value) => {
            context
                .write_instruction(Instruction::Const(value.value().clone().into()))
                .write_instruction(Instruction::ValEq)
                .cond_jump(on_fail);
        }
        ir::Value::Character(value) => {
            context
                .write_instruction(Instruction::Const((*value).into()))
                .write_instruction(Instruction::ValEq)
                .cond_jump(on_fail);
        }
        ir::Value::String(value) => {
            context
                .write_instruction(Instruction::Const(value.into()))
                .write_instruction(Instruction::ValEq)
                .cond_jump(on_fail);
        }
        ir::Value::Bits(value) => {
            context
                .write_instruction(Instruction::Const(value.value().clone().into()))
                .write_instruction(Instruction::ValEq)
                .cond_jump(on_fail);
        }
        ir::Value::Boolean(value) => {
            context
                .write_instruction(Instruction::Const((*value).into()))
                .write_instruction(Instruction::ValEq)
                .cond_jump(on_fail);
        }
        ir::Value::Unit => {
            context
                .write_instruction(Instruction::Const(Value::Unit))
                .write_instruction(Instruction::ValEq)
                .cond_jump(on_fail);
        }
        ir::Value::Conjunction(conj) => {
            let cleanup = context.labeler.unique_hint("conj_cleanup");
            let done = context.labeler.unique_hint("conj_done");
            context.write_instruction(Instruction::Copy);
            context.scope.intermediate();
            write_pattern_match(context, &conj.0, &cleanup);
            context.scope.end_intermediate();
            write_pattern_match(context, &conj.1, on_fail);
            context
                .jump(&done)
                .write_label(cleanup)
                .write_instruction(Instruction::Pop)
                .jump(on_fail)
                .write_label(done);
        }
        ir::Value::Disjunction(disj) => {
            let next = context.labeler.unique();
            let recover = context.labeler.unique_hint("disj2");
            context.write_instruction(Instruction::Copy);
            write_pattern_match(context, &disj.0, &recover);
            context
                .write_instruction(Instruction::Pop)
                .jump(&next)
                .write_label(recover);
            write_pattern_match(context, &disj.1, on_fail);
            context.write_label(next);
        }
        ir::Value::Wildcard => {
            context.write_instruction(Instruction::Pop);
        }
        ir::Value::Atom(value) => {
            let atom = context.atom(value);
            context
                .write_instruction(Instruction::Const(atom.into()))
                .write_instruction(Instruction::ValEq)
                .cond_jump(on_fail);
        }
        ir::Value::Reference(ident) => match context.scope.lookup(&ident.id).unwrap() {
            Binding::Variable(offset) => {
                let compare = context.labeler.unique_hint("compare");
                let assigned = context.labeler.unique_hint("assigned");
                context
                    .write_instruction(Instruction::Copy)
                    .write_instruction(Instruction::InitLocal(offset))
                    .cond_jump(&compare)
                    .write_instruction(Instruction::Pop)
                    .jump(&assigned)
                    .write_label(compare)
                    .write_instruction(Instruction::LoadLocal(offset))
                    .write_instruction(Instruction::ValEq)
                    .cond_jump(on_fail)
                    .write_label(assigned);
            }
            Binding::Static(..) => unreachable!(),
        },
        ir::Value::Application(application) => match unapply_2(application) {
            (None, ir::Value::Builtin(Builtin::Negate), value) => {
                let end = context.labeler.unique_hint("negate_end");
                let cleanup = context.labeler.unique_hint("negate_cleanup");
                context
                    .write_instruction(Instruction::Copy)
                    .write_instruction(Instruction::TypeOf)
                    .write_instruction(Instruction::Const("number".into()))
                    .write_instruction(Instruction::ValEq)
                    .cond_jump(&cleanup)
                    .write_instruction(Instruction::Negate);
                write_pattern(context, value, on_fail);
                context
                    .jump(&end)
                    .write_label(cleanup)
                    .jump(on_fail)
                    .write_instruction(Instruction::Pop)
                    .write_label(end);
            }
            (None, ir::Value::Builtin(Builtin::Pin), value) => {
                write_evaluation(context, value);
                context
                    .write_instruction(Instruction::ValEq)
                    .cond_jump(on_fail);
            }
            (Some(ir::Value::Builtin(Builtin::Glue)), lhs @ ir::Value::String(..), rhs) => {
                let end = context.labeler.unique_hint("glue_end");
                let cleanup = context.labeler.unique_hint("glue_cleanup");
                let double_cleanup = context.labeler.unique_hint("glue_cleanup2");
                context
                    .write_instruction(Instruction::Copy)
                    .write_instruction(Instruction::TypeOf)
                    .write_instruction(Instruction::Const("string".into()))
                    .write_instruction(Instruction::ValEq)
                    .cond_jump(&cleanup);
                let original = context.scope.intermediate();
                write_evaluation(context, lhs);
                let lhs_val = context.scope.intermediate();
                context
                    .write_instruction(Instruction::Copy)
                    .write_instruction(Instruction::Length)
                    .write_instruction(Instruction::LoadLocal(original))
                    .write_instruction(Instruction::Swap)
                    .write_instruction(Instruction::Take)
                    .write_instruction(Instruction::LoadLocal(lhs_val))
                    .write_instruction(Instruction::ValEq)
                    .cond_jump(&double_cleanup)
                    .write_instruction(Instruction::Length)
                    .write_instruction(Instruction::Skip);
                context.scope.end_intermediate();
                context.scope.end_intermediate();
                write_pattern(context, rhs, on_fail);
                context
                    .jump(&end)
                    .write_label(double_cleanup)
                    .write_instruction(Instruction::Pop)
                    .write_label(cleanup)
                    .write_instruction(Instruction::Pop)
                    .jump(on_fail)
                    .write_label(end);
            }
            (Some(ir::Value::Builtin(Builtin::Glue)), lhs, rhs @ ir::Value::String(..)) => {
                let end = context.labeler.unique_hint("glue_end");
                let cleanup = context.labeler.unique_hint("glue_cleanup");
                let double_cleanup = context.labeler.unique_hint("glue_cleanup2");
                context
                    .write_instruction(Instruction::Copy)
                    .write_instruction(Instruction::TypeOf)
                    .write_instruction(Instruction::Const("string".into()))
                    .write_instruction(Instruction::ValEq)
                    .cond_jump(&cleanup);
                let original = context.scope.intermediate();
                write_evaluation(context, rhs);
                let rhs_val = context.scope.intermediate();
                context
                    .write_instruction(Instruction::Copy)
                    .write_instruction(Instruction::Length)
                    .write_instruction(Instruction::LoadLocal(original))
                    .write_instruction(Instruction::Length)
                    .write_instruction(Instruction::Swap)
                    .write_instruction(Instruction::Subtract)
                    .write_instruction(Instruction::LoadLocal(original))
                    .write_instruction(Instruction::Swap)
                    .write_instruction(Instruction::Skip)
                    .write_instruction(Instruction::LoadLocal(rhs_val))
                    .write_instruction(Instruction::ValEq)
                    .cond_jump(&double_cleanup)
                    .write_instruction(Instruction::Length)
                    .write_instruction(Instruction::LoadLocal(original))
                    .write_instruction(Instruction::Length)
                    .write_instruction(Instruction::Swap)
                    .write_instruction(Instruction::Subtract)
                    .write_instruction(Instruction::Take);
                context.scope.end_intermediate();
                context.scope.end_intermediate();
                write_pattern(context, lhs, on_fail);
                context
                    .jump(&end)
                    .write_label(double_cleanup)
                    .write_instruction(Instruction::Pop)
                    .write_label(cleanup)
                    .write_instruction(Instruction::Pop)
                    .jump(on_fail)
                    .write_label(end);
            }
            (Some(ir::Value::Builtin(Builtin::Construct)), lhs, rhs) => {
                let cleanup = context.labeler.unique();
                context
                    .write_instruction(Instruction::Copy)
                    .write_instruction(Instruction::TypeOf)
                    .write_instruction(Instruction::Const("struct".into()))
                    .write_instruction(Instruction::ValEq)
                    .cond_jump(&cleanup)
                    .write_instruction(Instruction::Destruct)
                    .write_instruction(Instruction::Swap);
                // Match the atom, very easy
                write_pattern(context, lhs, &cleanup);
                // If the atom matching fails, we have to clean up the extra value
                let match_value = context.labeler.unique_hint("structvalue");
                context
                    .jump(&match_value)
                    .write_label(cleanup)
                    .write_instruction(Instruction::Pop)
                    .jump(on_fail)
                    .write_label(match_value);
                write_pattern(context, rhs, on_fail);
            }
            (Some(ir::Value::Builtin(Builtin::Cons)), lhs, rhs) => {
                let cleanup = context.labeler.unique();
                context
                    .write_instruction(Instruction::Copy)
                    .write_instruction(Instruction::TypeOf)
                    .write_instruction(Instruction::Const("tuple".into()))
                    .write_instruction(Instruction::ValEq)
                    .cond_jump(&cleanup)
                    .write_instruction(Instruction::Uncons)
                    .write_instruction(Instruction::Swap);
                write_pattern(context, lhs, &cleanup);
                // If the first matching fails, we have to clean up the second
                let match_second = context.labeler.unique_hint("snd");
                context
                    .jump(&match_second)
                    .write_label(cleanup)
                    .write_instruction(Instruction::Pop)
                    .jump(on_fail)
                    .write_label(match_second);
                write_pattern(context, rhs, on_fail);
            }
            (None, ir::Value::Builtin(Builtin::Array), ir::Value::Pack(pack)) => {
                let cleanup = context.labeler.unique_hint("array_cleanup");
                let end = context.labeler.unique_hint("array_end");
                let mut spread = None;
                context.write_instruction(Instruction::Clone);
                let array = context.scope.intermediate();
                for element in &pack.values {
                    if element.is_spread {
                        spread = Some(&element.expression);
                        continue;
                    }
                    // TODO: this could be way more efficient
                    if spread.is_none() {
                        context
                            .write_instruction(Instruction::Copy)
                            .write_instruction(Instruction::Const(0.into()))
                            .write_instruction(Instruction::Access);
                        write_pattern_match(context, &element.expression, &cleanup);
                        context
                            .write_instruction(Instruction::Const(1.into()))
                            .write_instruction(Instruction::Skip);
                    } else {
                        let index = context.scope.intermediate();
                        context
                            .write_instruction(Instruction::Copy)
                            .write_instruction(Instruction::Length)
                            .write_instruction(Instruction::Const(1.into()))
                            .write_instruction(Instruction::Subtract)
                            .write_instruction(Instruction::LoadLocal(array))
                            .write_instruction(Instruction::LoadLocal(index))
                            .write_instruction(Instruction::Access);
                        write_pattern_match(context, &element.expression, &cleanup);
                        context.write_instruction(Instruction::Take);
                        context.scope.end_intermediate();
                    }
                }
                if let Some(spread) = spread {
                    write_pattern_match(context, spread, on_fail);
                } else {
                    context.write_instruction(Instruction::Pop);
                }
                context.scope.end_intermediate();
                context
                    .jump(&end)
                    .write_label(cleanup)
                    .write_instruction(Instruction::Pop)
                    .write_instruction(Instruction::Pop)
                    .jump(on_fail)
                    .write_label(end);
            }
            (None, ir::Value::Builtin(Builtin::Record), ir::Value::Pack(pack)) => {
                let cleanup = context.labeler.unique_hint("record_cleanup");
                let end = context.labeler.unique_hint("record_end");
                let mut spread = None;
                context.write_instruction(Instruction::Clone);
                let record = context.scope.intermediate();
                for element in &pack.values {
                    if element.is_spread {
                        spread = Some(&element.expression);
                        continue;
                    }
                    let ir::Value::Mapping(mapping) = &element.expression.value else {
                        panic!("record pattern elements must be mapping ");
                    };
                    write_expression(context, &mapping.0);
                    let key = context.scope.intermediate();
                    context
                        .write_instruction(Instruction::LoadLocal(record))
                        .write_instruction(Instruction::LoadLocal(key))
                        .write_instruction(Instruction::Contains)
                        .cond_jump(&cleanup)
                        .write_instruction(Instruction::LoadLocal(record))
                        .write_instruction(Instruction::LoadLocal(key))
                        .write_instruction(Instruction::Access);
                    write_pattern_match(context, &mapping.1, &cleanup);
                    context.write_instruction(Instruction::Delete);
                    context.scope.end_intermediate();
                }
                if let Some(spread) = spread {
                    write_pattern_match(context, spread, on_fail);
                } else {
                    context.write_instruction(Instruction::Pop);
                }
                context.scope.end_intermediate();
                context
                    .jump(&end)
                    .write_label(cleanup)
                    .write_instruction(Instruction::Pop)
                    .write_instruction(Instruction::Pop)
                    .jump(on_fail)
                    .write_label(end);
            }
            (None, ir::Value::Builtin(Builtin::Set), ir::Value::Pack(pack)) => {
                let cleanup = context.labeler.unique_hint("set_cleanup");
                let end = context.labeler.unique_hint("set_end");
                let mut spread = None;
                context.write_instruction(Instruction::Clone);
                let set = context.scope.intermediate();
                for element in &pack.values {
                    if element.is_spread {
                        spread = Some(&element.expression);
                        continue;
                    }
                    write_expression(context, &element.expression);
                    let value = context.scope.intermediate();
                    context
                        .write_instruction(Instruction::LoadLocal(set))
                        .write_instruction(Instruction::LoadLocal(value))
                        .write_instruction(Instruction::Contains)
                        .cond_jump(&cleanup);
                    context.write_instruction(Instruction::Delete);
                    context.scope.end_intermediate();
                }
                if let Some(spread) = spread {
                    write_pattern_match(context, spread, on_fail);
                } else {
                    context.write_instruction(Instruction::Pop);
                }
                context.scope.end_intermediate();
                context
                    .jump(&end)
                    .write_label(cleanup)
                    .write_instruction(Instruction::Pop)
                    .write_instruction(Instruction::Pop)
                    .jump(on_fail)
                    .write_label(end);
            }
            what => panic!("not a pattern ({what:?})"),
        },
        ir::Value::Dynamic(dynamic) => {
            panic!("Dynamic is not actually supposed to happen, but we got {dynamic:?}");
        }
        value => panic!("{value:?} is not a pattern"),
    }
}
