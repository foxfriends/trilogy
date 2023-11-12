use crate::prelude::*;
use trilogy_ir::ir::{self, Builtin, Expression};
use trilogy_vm::Instruction;

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
        ir::Value::Mapping(..) => unreachable!(),
        ir::Value::Number(value) => {
            context
                .constant(value.value().clone())
                .instruction(Instruction::ValEq)
                .cond_jump(on_fail);
        }
        ir::Value::Character(value) => {
            context
                .constant(*value)
                .instruction(Instruction::ValEq)
                .cond_jump(on_fail);
        }
        ir::Value::String(value) => {
            context
                .constant(value)
                .instruction(Instruction::ValEq)
                .cond_jump(on_fail);
        }
        ir::Value::Bits(value) => {
            context
                .constant(value.value().clone())
                .instruction(Instruction::ValEq)
                .cond_jump(on_fail);
        }
        ir::Value::Boolean(value) => {
            context
                .constant(*value)
                .instruction(Instruction::ValEq)
                .cond_jump(on_fail);
        }
        ir::Value::Unit => {
            context
                .constant(())
                .instruction(Instruction::ValEq)
                .cond_jump(on_fail);
        }
        ir::Value::Conjunction(conj) => {
            let cleanup = context.make_label("conj_cleanup");
            context.instruction(Instruction::Copy);
            context.scope.intermediate();
            write_pattern_match(context, &conj.0, &cleanup);
            context.scope.end_intermediate();
            write_pattern_match(context, &conj.1, on_fail);
            context.bubble(|c| {
                c.label(cleanup).instruction(Instruction::Pop).jump(on_fail);
            });
        }
        ir::Value::Disjunction(disj) => {
            let recover = context.make_label("disj2");
            context.instruction(Instruction::Copy);
            write_pattern_match(context, &disj.0, &recover);
            context.instruction(Instruction::Pop).bubble(|c| {
                c.label(recover);
                write_pattern_match(c, &disj.1, on_fail);
            });
        }
        ir::Value::Wildcard => {
            context.instruction(Instruction::Pop);
        }
        ir::Value::Atom(value) => {
            context
                .atom(value)
                .instruction(Instruction::ValEq)
                .cond_jump(on_fail);
        }
        ir::Value::Reference(ident) => match context.scope.lookup(&ident.id).unwrap() {
            Binding::Variable(offset) => {
                let compare = context.make_label("compare");
                context
                    .instruction(Instruction::Copy)
                    .instruction(Instruction::InitLocal(offset))
                    .cond_jump(&compare)
                    .instruction(Instruction::Pop)
                    .bubble(|c| {
                        c.label(compare)
                            .instruction(Instruction::LoadLocal(offset))
                            .instruction(Instruction::ValEq)
                            .cond_jump(on_fail);
                    });
            }
            Binding::Static(..) | Binding::Context(..) => {
                unreachable!("this is a new binding, so it cannot be static")
            }
        },
        ir::Value::Application(application) => match unapply_2(application) {
            (None, ir::Value::Builtin(Builtin::Negate), value) => {
                let cleanup = context.make_label("negate_cleanup");
                context
                    .try_type("number", Err(&cleanup))
                    .instruction(Instruction::Negate);
                write_pattern(context, value, on_fail);
                context.bubble(|c| {
                    c.label(cleanup).jump(on_fail).instruction(Instruction::Pop);
                });
            }
            (None, ir::Value::Builtin(Builtin::Pin), value) => {
                write_evaluation(context, value);
                context.instruction(Instruction::ValEq).cond_jump(on_fail);
            }
            (Some(ir::Value::Builtin(Builtin::Glue)), lhs @ ir::Value::String(..), rhs) => {
                let cleanup = context.make_label("glue_cleanup");
                let double_cleanup = context.make_label("glue_cleanup2");
                context.try_type("string", Err(&cleanup));
                let original = context.scope.intermediate();
                write_evaluation(context, lhs);
                let lhs_val = context.scope.intermediate();
                context
                    .instruction(Instruction::Copy)
                    .instruction(Instruction::Length)
                    .instruction(Instruction::LoadLocal(original))
                    .instruction(Instruction::Swap)
                    .instruction(Instruction::Take)
                    .instruction(Instruction::LoadLocal(lhs_val))
                    .instruction(Instruction::ValEq)
                    .cond_jump(&double_cleanup)
                    .instruction(Instruction::Length)
                    .instruction(Instruction::Skip);
                context.scope.end_intermediate();
                context.scope.end_intermediate();
                write_pattern(context, rhs, on_fail);
                context.bubble(|c| {
                    c.label(double_cleanup)
                        .instruction(Instruction::Pop)
                        .label(cleanup)
                        .instruction(Instruction::Pop)
                        .jump(on_fail);
                });
            }
            (Some(ir::Value::Builtin(Builtin::Glue)), lhs, rhs @ ir::Value::String(..)) => {
                let cleanup = context.make_label("glue_cleanup");
                let double_cleanup = context.make_label("glue_cleanup2");
                context.try_type("string", Err(&cleanup));
                let original = context.scope.intermediate();
                write_evaluation(context, rhs);
                let rhs_val = context.scope.intermediate();
                context
                    .instruction(Instruction::Copy)
                    .instruction(Instruction::Length)
                    .instruction(Instruction::LoadLocal(original))
                    .instruction(Instruction::Length)
                    .instruction(Instruction::Swap)
                    .instruction(Instruction::Subtract)
                    .instruction(Instruction::LoadLocal(original))
                    .instruction(Instruction::Swap)
                    .instruction(Instruction::Skip)
                    .instruction(Instruction::LoadLocal(rhs_val))
                    .instruction(Instruction::ValEq)
                    .cond_jump(&double_cleanup)
                    .instruction(Instruction::Length)
                    .instruction(Instruction::LoadLocal(original))
                    .instruction(Instruction::Length)
                    .instruction(Instruction::Swap)
                    .instruction(Instruction::Subtract)
                    .instruction(Instruction::Take);
                context.scope.end_intermediate();
                context.scope.end_intermediate();
                write_pattern(context, lhs, on_fail);
                context.bubble(|c| {
                    c.label(double_cleanup)
                        .instruction(Instruction::Pop)
                        .label(cleanup)
                        .instruction(Instruction::Pop)
                        .jump(on_fail);
                });
            }
            (Some(ir::Value::Builtin(Builtin::Construct)), lhs, rhs) => {
                let cleanup = context.make_label("cleanup");
                context
                    .try_type("struct", Err(&cleanup))
                    .instruction(Instruction::Destruct);
                // Match the atom
                write_pattern(context, rhs, &cleanup);
                // Then match the contents
                write_pattern(context, lhs, on_fail);
                context.bubble(|c| {
                    // If the atom matching fails, we have to clean up the extra value
                    c.label(cleanup).instruction(Instruction::Pop).jump(on_fail);
                });
            }
            (Some(ir::Value::Builtin(Builtin::Cons)), lhs, rhs) => {
                let cleanup = context.make_label("cleanup");
                context
                    .try_type("tuple", Err(&cleanup))
                    .instruction(Instruction::Uncons)
                    .instruction(Instruction::Swap);
                write_pattern(context, lhs, &cleanup);
                write_pattern(context, rhs, on_fail);
                // If the first matching fails, we have to clean up the second
                context.bubble(|c| {
                    c.label(cleanup).instruction(Instruction::Pop).jump(on_fail);
                });
            }
            (None, ir::Value::Builtin(Builtin::Array), ir::Value::Pack(pack)) => {
                let cleanup = context.make_label("array_cleanup");
                // Before even attempting to match this array, check its length and the length of
                // the pattern. If the pattern is longer than the array, then give up already.
                // The spread element doesn't count towards length since it can be 0. If the pattern
                // is shorter than the array and there is no spread, then also give up.
                let needed = pack
                    .values
                    .iter()
                    .filter(|element| !element.is_spread)
                    .count();
                let cmp = if pack.values.iter().any(|el| el.is_spread) {
                    Instruction::Geq
                } else {
                    Instruction::ValEq
                };
                context
                    .try_type("array", Err(&cleanup))
                    .instruction(Instruction::Copy)
                    .instruction(Instruction::Length)
                    .constant(needed)
                    .instruction(cmp)
                    .cond_jump(&cleanup);
                // If that worked, then we'll have enough elements and won't have to check that
                // below at all.

                // Going to be modifying this array in place, so clone it before we begin.
                // Trilogy does not have slices.
                context.instruction(Instruction::Clone);
                let array = context.scope.intermediate();
                for (i, element) in pack.values.iter().enumerate() {
                    if element.is_spread {
                        // When it's the spread element, take all the elements we aren't going to
                        // need for the tail of this pattern from the array.
                        let next = context.make_label("spread_next");
                        let cleanup_spread = context.make_label("cleanup_spread");
                        let elements_in_tail = pack.values.len() - i - 1;
                        context
                            // First determine the runtime length to find out how many elements
                            // we don't need later.
                            .instruction(Instruction::Copy)
                            .instruction(Instruction::Length)
                            .constant(elements_in_tail)
                            .instruction(Instruction::Subtract);
                        let length = context.scope.intermediate();
                        context
                            // Take that many elements from (a copy of) the array
                            .instruction(Instruction::LoadLocal(array))
                            .instruction(Instruction::LoadLocal(length))
                            .instruction(Instruction::Take);
                        // And match that prefix with the element pattern
                        write_pattern_match(context, &element.expression, &cleanup_spread);
                        // Then use the copy of length that's still on the stack to drop those
                        // elements we just took from the original array.
                        context.instruction(Instruction::Skip).jump(&next);
                        // If we fail during the spread matching, the length that's on the stack has
                        // to be discarded still.
                        context
                            .label(cleanup_spread)
                            .instruction(Instruction::Pop)
                            .jump(&cleanup)
                            .label(next);
                        context.scope.end_intermediate(); // length
                    } else {
                        // When it's not the spread element, just match the first element.
                        context
                            .instruction(Instruction::Copy)
                            .constant(0)
                            .instruction(Instruction::Access);
                        write_pattern_match(context, &element.expression, &cleanup);
                        // And then we drop that element from the array and leave just the tail on
                        // the stack.
                        context.constant(1).instruction(Instruction::Skip);
                    }
                }
                // There should now be an empty array on the stack, so get rid of it before continuing.
                context.instruction(Instruction::Pop);
                context.bubble(|c| {
                    c.label(cleanup)
                        // Otherwise, we have to cleanup. The only thing on the stack is the array.
                        .instruction(Instruction::Pop)
                        .jump(on_fail);
                });
                context.scope.end_intermediate(); // array
            }
            (None, ir::Value::Builtin(Builtin::Record), ir::Value::Pack(pack)) => {
                let cleanup1 = context.make_label("record_cleanup1");
                let cleanup2 = context.make_label("record_cleanup2");
                let mut spread = None;
                context
                    .try_type("record", Err(&cleanup1))
                    .instruction(Instruction::Clone);
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
                        .instruction(Instruction::LoadLocal(record))
                        .instruction(Instruction::LoadLocal(key))
                        .instruction(Instruction::Contains)
                        .cond_jump(&cleanup2)
                        .instruction(Instruction::LoadLocal(record))
                        .instruction(Instruction::LoadLocal(key))
                        .instruction(Instruction::Access);
                    write_pattern_match(context, &mapping.1, &cleanup2);
                    context.instruction(Instruction::Delete);
                    context.scope.end_intermediate();
                }
                context.scope.end_intermediate();
                if let Some(spread) = spread {
                    write_pattern_match(context, spread, on_fail);
                } else {
                    context
                        .instruction(Instruction::Length)
                        .constant(0)
                        .instruction(Instruction::ValEq)
                        .cond_jump(on_fail);
                }
                context.bubble(|c| {
                    c.label(cleanup2)
                        .instruction(Instruction::Pop)
                        .label(cleanup1)
                        .instruction(Instruction::Pop)
                        .jump(on_fail);
                });
            }
            (None, ir::Value::Builtin(Builtin::Set), ir::Value::Pack(pack)) => {
                let cleanup1 = context.make_label("set_cleanup1");
                let cleanup2 = context.make_label("set_cleanup2");
                let mut spread = None;
                context
                    .try_type("set", Err(&cleanup1))
                    .instruction(Instruction::Clone);
                let set = context.scope.intermediate();
                for element in &pack.values {
                    if element.is_spread {
                        spread = Some(&element.expression);
                        continue;
                    }
                    write_expression(context, &element.expression);
                    let value = context.scope.intermediate();
                    context
                        .instruction(Instruction::LoadLocal(set))
                        .instruction(Instruction::LoadLocal(value))
                        .instruction(Instruction::Contains)
                        .cond_jump(&cleanup2)
                        .instruction(Instruction::Delete);
                    context.scope.end_intermediate();
                }
                context.scope.end_intermediate();
                if let Some(spread) = spread {
                    write_pattern_match(context, spread, on_fail);
                } else {
                    context
                        .instruction(Instruction::Length)
                        .constant(0)
                        .instruction(Instruction::ValEq)
                        .cond_jump(on_fail);
                }
                context.bubble(|c| {
                    c.label(cleanup2)
                        .instruction(Instruction::Pop)
                        .label(cleanup1)
                        .instruction(Instruction::Pop)
                        .jump(on_fail);
                });
            }
            what => panic!("not a pattern ({what:?})"),
        },
        ir::Value::Dynamic(dynamic) => {
            panic!("Dynamic is not actually supposed to happen, but we got {dynamic:?}");
        }
        value => panic!("{value:?} is not a pattern"),
    }
}
