use crate::preamble::{END, RETURN};
use crate::{prelude::*, ASSIGN};
use trilogy_ir::ir::{self, Iterator};
use trilogy_ir::visitor::HasBindings;
use trilogy_vm::{Array, Instruction, Offset, Record, Set, Value};

#[inline(always)]
pub(crate) fn write_expression(context: &mut Context, expr: &ir::Expression) {
    write_evaluation(context, &expr.value)
}

pub(crate) fn write_evaluation(context: &mut Context, value: &ir::Value) {
    match &value {
        ir::Value::Builtin(builtin) if is_referenceable_operator(*builtin) => {
            write_operator_reference(context, *builtin);
        }
        ir::Value::Builtin(builtin) => panic!("{builtin:?} is not a referenceable builtin"),
        ir::Value::Pack(pack) => {
            for element in &pack.values {
                if element.is_spread {
                    panic!("spread elements are not available in generalized packs");
                }
                write_expression(context, &element.expression);
            }
        }
        ir::Value::Sequence(seq) => {
            let mut seq = seq.iter();
            let Some(mut expr) = seq.next() else {
                // An empty sequence must still have a value
                context.constant(());
                return;
            };
            loop {
                write_expression(context, expr);
                let Some(next_expr) = seq.next() else { break };
                expr = next_expr;
                context.instruction(Instruction::Pop);
            }
        }
        ir::Value::Assignment(assignment) => match &assignment.lhs.value {
            ir::Value::Reference(var) => {
                write_expression(context, &assignment.rhs);
                match context.scope.lookup(&var.id) {
                    Some(Binding::Variable(index)) => {
                        context
                            .instruction(Instruction::Copy)
                            .instruction(Instruction::SetLocal(index));
                    }
                    _ => unreachable!("Only variables can be assigned to"),
                }
            }
            ir::Value::Application(application) => match unapply_2(application) {
                (Some(ir::Value::Builtin(ir::Builtin::Access)), collection, key) => {
                    context.reference(ASSIGN);
                    write_evaluation(context, collection);
                    context.scope.intermediate();
                    write_evaluation(context, key);
                    context.scope.intermediate();
                    write_expression(context, &assignment.rhs);
                    context.scope.end_intermediate();
                    context.scope.end_intermediate();
                    context.call_procedure(3);
                }
                _ => unreachable!("LValue applications must be access"),
            },
            _ => unreachable!("LValues must be reference or application"),
        },
        ir::Value::Mapping(value) => {
            write_expression(context, &value.0);
            context.scope.intermediate();
            write_expression(context, &value.1);
            context.scope.end_intermediate();
        }
        ir::Value::Number(value) => {
            context.constant(value.value().clone());
        }
        ir::Value::Character(value) => {
            context.constant(*value);
        }
        ir::Value::String(value) => {
            context.constant(value);
        }
        ir::Value::Bits(value) => {
            context.constant(value.value().clone());
        }
        ir::Value::Boolean(value) => {
            context.constant(*value);
        }
        ir::Value::Unit => {
            context.instruction(Instruction::Const(Value::Unit));
        }
        ir::Value::Conjunction(..) => unreachable!("Conjunction cannot appear in an evaluation"),
        ir::Value::Disjunction(..) => unreachable!("Disjunction cannot appear in an evaluation"),
        ir::Value::Wildcard => unreachable!("Wildcard cannot appear in an evaluation"),
        ir::Value::Atom(value) => {
            context.atom(value);
        }
        ir::Value::Query(..) => unreachable!("Query cannot appear in an evaluation"),
        ir::Value::Iterator(iterator) => write_iterator(context, iterator, None, None),
        ir::Value::While(stmt) => {
            let begin = context.make_label("while");
            let cleanup = context.make_label("while_cleanup");
            let end = context.make_label("while_end");

            // Break and continue are just regular intermediates at first
            let r#break = context.r#break(&end).intermediate();
            let r#continue = context.r#continue(&begin).intermediate();

            // The actual loop we can implement in the standard way after the continuations are
            // created.
            context.label(&begin);
            // Check the condition
            write_expression(context, &stmt.condition);
            context.typecheck("boolean").cond_jump(&cleanup);
            // It's only in the body of the loop that continue and break become usable,
            // so we only make them referenceable here
            context.scope.push_break(r#break);
            context.scope.push_continue(r#continue);
            // If it's true, run the body. The body has access to continue and break.
            write_expression(context, &stmt.body);
            context
                .instruction(Instruction::Pop)
                .jump(&begin)
                .label(&cleanup)
                .instruction(Instruction::Pop) // continue
                .end_intermediate()
                .instruction(Instruction::Pop) // break
                .end_intermediate()
                .label(&end)
                .constant(()); // The assumed value on stack at end of evaluation
            context.scope.pop_break();
            context.scope.pop_continue();
        }
        ir::Value::Application(application) => match unapply_2(application) {
            (
                Some(ir::Value::Builtin(ir::Builtin::ModuleAccess)),
                module_ref,
                ir::Value::Dynamic(ident),
            ) => {
                write_evaluation(context, module_ref);
                context.typecheck("callable").atom(&**ident).call_module();
            }
            (None, ir::Value::Builtin(builtin), arg) if is_unary_operator(*builtin) => {
                write_evaluation(context, arg);
                write_operator(context, *builtin);
            }
            (Some(ir::Value::Builtin(builtin)), lhs, rhs) if is_operator(*builtin) => {
                write_evaluation(context, lhs);
                context.scope.intermediate();
                write_evaluation(context, rhs);
                context.scope.end_intermediate();
                write_operator(context, *builtin);
            }
            (None, ir::Value::Builtin(ir::Builtin::Record), ir::Value::Pack(pack)) => {
                context.constant(Record::default());
                context.scope.intermediate();
                for element in &pack.values {
                    write_expression(context, &element.expression);
                    if element.is_spread {
                        context.typecheck("record").instruction(Instruction::Glue);
                    } else {
                        context.instruction(Instruction::Assign);
                    }
                }
                context.scope.end_intermediate();
            }
            (None, ir::Value::Builtin(ir::Builtin::Record), arg @ ir::Value::Iterator(..)) => {
                write_evaluation(context, arg);
                let iterator = context.scope.intermediate();
                context.constant(Record::default());
                context.scope.intermediate();
                context
                    .repeat(|c, exit| {
                        c.instruction(Instruction::LoadLocal(iterator))
                            .iterate(exit)
                            // Between computing the next value and inserting it, we have to clone
                            // the collection due to the potential for the call to the iterator to
                            // return multiple times. In each parallel execution, the iterator must
                            // collect into a separate array. In the single-execution case, this adds
                            // a decent amount of overhead... so hopefully an alternative can be found
                            .instruction(Instruction::Swap)
                            .instruction(Instruction::Clone)
                            .instruction(Instruction::Swap)
                            .instruction(Instruction::Uncons)
                            .instruction(Instruction::Assign);
                    })
                    .instruction(Instruction::Pop)
                    .instruction(Instruction::Swap)
                    .instruction(Instruction::Pop);
                context.scope.end_intermediate();
                context.scope.end_intermediate();
            }
            (None, ir::Value::Builtin(ir::Builtin::Record), ..) => {
                unreachable!("record is applied to pack or iterator");
            }
            (None, ir::Value::Builtin(ir::Builtin::Set), ir::Value::Pack(pack)) => {
                context.constant(Set::default());
                context.scope.intermediate();
                for element in &pack.values {
                    write_expression(context, &element.expression);
                    if element.is_spread {
                        context.typecheck("set").instruction(Instruction::Glue);
                    } else {
                        context.instruction(Instruction::Insert);
                    }
                }
                context.scope.end_intermediate();
            }
            (None, ir::Value::Builtin(ir::Builtin::Set), arg @ ir::Value::Iterator(..)) => {
                write_evaluation(context, arg);
                let iterator = context.scope.intermediate();
                context.constant(Set::default());
                context.scope.intermediate();
                context
                    .repeat(|c, exit| {
                        c.instruction(Instruction::LoadLocal(iterator))
                            .iterate(exit)
                            // Between computing the next value and inserting it, we have to clone
                            // the collection due to the potential for the call to the iterator to
                            // return multiple times. In each parallel execution, the iterator must
                            // collect into a separate array. In the single-execution case, this adds
                            // a decent amount of overhead... so hopefully an alternative can be found
                            .instruction(Instruction::Swap)
                            .instruction(Instruction::Clone)
                            .instruction(Instruction::Swap)
                            .instruction(Instruction::Insert);
                    })
                    .instruction(Instruction::Pop)
                    .instruction(Instruction::Swap)
                    .instruction(Instruction::Pop);
                context.scope.end_intermediate();
                context.scope.end_intermediate();
            }
            (None, ir::Value::Builtin(ir::Builtin::Set), ..) => {
                unreachable!("set is applied to pack or iterator");
            }
            (None, ir::Value::Builtin(ir::Builtin::Array), ir::Value::Pack(pack)) => {
                context.constant(Array::default());
                context.scope.intermediate();
                for element in &pack.values {
                    context.instruction(Instruction::Clone);
                    write_expression(context, &element.expression);
                    if element.is_spread {
                        context.typecheck("array").instruction(Instruction::Glue);
                    } else {
                        context.instruction(Instruction::Insert);
                    }
                }
                context.scope.end_intermediate();
            }
            (None, ir::Value::Builtin(ir::Builtin::Array), arg @ ir::Value::Iterator(..)) => {
                write_evaluation(context, arg);
                let iterator = context.intermediate();

                context.constant(Array::default()).intermediate();

                context
                    .repeat(|context, exit| {
                        context
                            .instruction(Instruction::LoadLocal(iterator))
                            .iterate(exit)
                            // Between computing the next value and inserting it, we have to clone
                            // the collection due to the potential for the call to the iterator to
                            // return multiple times. In each parallel execution, the iterator must
                            // collect into a separate array. In the single-execution case, this adds
                            // a decent amount of overhead... so hopefully an alternative can be found
                            .instruction(Instruction::Swap)
                            .instruction(Instruction::Clone)
                            .instruction(Instruction::Swap)
                            .instruction(Instruction::Insert);
                    })
                    .instruction(Instruction::Pop) // 'done
                    .instruction(Instruction::Swap)
                    .end_intermediate() // array
                    .instruction(Instruction::Pop)
                    .end_intermediate(); // iterator
            }
            (None, ir::Value::Builtin(ir::Builtin::Array), ..) => {
                unreachable!("array is applied to pack or iterator");
            }
            (None, ir::Value::Builtin(ir::Builtin::For), ir::Value::Iterator(value)) => {
                let end = context.make_label("for_end");
                let cleanup = context.make_label("for_cleanup");
                let continued = context.make_label("for_next");
                let broke = context.make_label("for_broke");
                let begin = context.make_label("for");

                // Make a spot on the stack for the "return value" of this for loop. For loops
                // internally evaluate to true if they iterate at all, or false if there were
                // no iterations taken. That boolean gets used by the if statements generated
                // by the `for .. else` syntax transformation.
                let eval_to = context.constant(false).intermediate();
                let iterator = context.instruction(Instruction::Variable).intermediate();
                let r#break = context.r#break(&broke).intermediate();
                let r#continue = context.r#continue(&continued).intermediate();
                write_iterator(context, value, Some(r#continue), Some(r#break));
                context
                    .instruction(Instruction::SetLocal(iterator))
                    .label(&begin)
                    .instruction(Instruction::LoadLocal(iterator))
                    .iterate(&cleanup)
                    .instruction(Instruction::Pop) // the value of the loop body does not matter
                    .label(&continued)
                    .constant(true)
                    .instruction(Instruction::SetLocal(eval_to))
                    .jump(&begin)
                    .label(&cleanup)
                    .instruction(Instruction::Pop) // 'done
                    .instruction(Instruction::Pop)
                    .end_intermediate() // continue
                    .instruction(Instruction::Pop)
                    .end_intermediate() // break
                    .bubble(|c| {
                        c.label(&broke)
                            .constant(true)
                            .instruction(Instruction::SetLocal(eval_to));
                    })
                    .label(&end)
                    .instruction(Instruction::Pop)
                    .end_intermediate() // iterator
                    .end_intermediate(); // eval_to
            }
            (None, ir::Value::Builtin(ir::Builtin::Is), ir::Value::Query(query)) => {
                let is_fail = context.make_label("is_fail");
                let var_count = context.declare_variables(query.bindings());
                write_query_state(context, query);
                write_query(context, query, &is_fail);
                context
                    .constant(true)
                    .bubble(|c| {
                        c.label(&is_fail).constant(false);
                    })
                    .instruction(Instruction::Slide(var_count as u32 + 1));
                context.undeclare_variables(query.bindings(), false);
                for _ in 0..=var_count {
                    // One extra POP to discard the query state
                    context.instruction(Instruction::Pop);
                }
            }
            _ => {
                write_expression(context, &application.function);
                context.typecheck("callable");
                context.scope.intermediate();
                write_expression(context, &application.argument);
                context.scope.end_intermediate();
                match &application.argument.value {
                    ir::Value::Pack(pack) => {
                        let arity = pack
                            .len()
                            .expect("procedures may not have spread arguments");
                        context.call_procedure(arity);
                    }
                    _ => {
                        context.call_function();
                    }
                };
            }
        },
        ir::Value::Let(decl) if decl.query.is_once() => {
            let reenter = context.make_label("let");
            let declared = context.declare_variables(decl.query.bindings());
            write_query_state(context, &decl.query);
            context.label(reenter.clone());
            write_query(context, &decl.query, END);
            // After running the query, we don't need the state anymore
            context.instruction(Instruction::Pop);
            write_expression(context, &decl.body);
            context.instruction(Instruction::Slide(declared as u32));
            // After the body has been executed, the variables bound in the query are dropped
            context.undeclare_variables(decl.query.bindings(), true);
        }
        ir::Value::Let(decl) => {
            let reenter = context.make_label("let");
            let declared = context.declare_variables(decl.query.bindings());
            write_query_state(context, &decl.query);
            context.label(reenter.clone());
            write_query(context, &decl.query, END);
            context.scope.intermediate();
            context
                .instruction(Instruction::Const(Value::Bool(true)))
                .instruction(Instruction::Const(Value::Bool(false)))
                .instruction(Instruction::Branch)
                .cond_jump(&reenter)
                .instruction(Instruction::Pop);
            context.scope.end_intermediate();
            write_expression(context, &decl.body);
            context.instruction(Instruction::Slide(declared as u32));
            context.undeclare_variables(decl.query.bindings(), true);
        }
        ir::Value::IfElse(cond) => {
            let when_false = context.make_label("else");
            write_expression(context, &cond.condition);
            context.typecheck("boolean").cond_jump(&when_false);
            write_expression(context, &cond.when_true);
            let end = context.make_label("end_if");
            context.jump(&end);
            context.label(when_false);
            write_expression(context, &cond.when_false);
            context.label(end);
        }
        ir::Value::Match(cond) => {
            write_expression(context, &cond.expression);
            let val = context.scope.intermediate();
            let end = context.make_label("match_end");
            for case in &cond.cases {
                let cleanup = context.make_label("case_cleanup");
                let vars = context.declare_variables(case.pattern.bindings());
                context.instruction(Instruction::LoadLocal(val));
                write_pattern_match(context, &case.pattern, &cleanup);
                write_expression(context, &case.guard);
                context.typecheck("boolean").cond_jump(&cleanup);
                write_expression(context, &case.body);
                context.instruction(Instruction::SetLocal(val));
                context.undeclare_variables(case.pattern.bindings(), true);
                context.jump(&end);
                context.label(cleanup);
                for _ in 0..vars {
                    context.instruction(Instruction::Pop);
                }
            }
            context.scope.end_intermediate();
            context.label(end);
        }
        ir::Value::Fn(closure) => {
            let end = context.make_label("end_fn");
            let params = context.scope.closure(closure.parameters.len());
            for i in 0..closure.parameters.len() {
                context
                    .close(if i == 0 { &end } else { RETURN })
                    .unlock_function();
            }
            for (i, parameter) in closure.parameters.iter().enumerate() {
                context.declare_variables(parameter.bindings());
                context.instruction(Instruction::LoadLocal(params + i as u32));
                write_pattern_match(context, parameter, END);
            }
            write_expression(context, &closure.body);
            for parameter in closure.parameters.iter().rev() {
                context.undeclare_variables(parameter.bindings(), false);
                context.scope.unclosure(1)
            }

            context.label(end);
        }
        ir::Value::Do(closure) => {
            let arity = closure.parameters.len();
            let param_start = context.scope.closure(arity);
            context.proc_closure(arity, |context| {
                for (offset, parameter) in closure.parameters.iter().enumerate() {
                    context.declare_variables(parameter.bindings());
                    context.instruction(Instruction::LoadLocal(param_start + offset as u32));
                    write_pattern_match(context, parameter, END);
                }
                write_expression(context, &closure.body);
                context
                    .instruction(Instruction::Const(Value::Unit))
                    .instruction(Instruction::Return);
                for parameter in closure.parameters.iter().rev() {
                    context.undeclare_variables(parameter.bindings(), false);
                }
            });
            context.scope.unclosure(arity);
        }
        ir::Value::Handled(handled) => {
            context.with(
                |context, effect| {
                    for handler in &handled.handlers {
                        context
                            .case(|context, next| {
                                context.declare_variables(handler.pattern.bindings());
                                context.instruction(Instruction::LoadLocal(effect));
                                write_pattern_match(context, &handler.pattern, next);
                                write_expression(context, &handler.guard);
                                context.typecheck("boolean").cond_jump(next);
                                write_expression(context, &handler.body);
                                // At the end of an effect handler, everything just ends.
                                //
                                // The effect handler should have used cancel or resume appropriately
                                // if "just end" was not the intention.
                                context.instruction(Instruction::Fizzle);
                            })
                            .undeclare_variables(handler.pattern.bindings(), true);
                    }
                },
                |context| {
                    write_expression(context, &handled.expression);
                },
            );
        }
        ir::Value::Reference(ident) => {
            let binding = context
                .scope
                .lookup(&ident.id)
                .expect("unresolved reference should not exist at this point");
            match binding {
                Binding::Variable(offset) => {
                    context.instruction(Instruction::LoadLocal(offset));
                }
                Binding::Static(label) => {
                    context.reference(label.to_owned());
                }
                Binding::Context(label) => {
                    context
                        .reference(label.to_owned())
                        .instruction(Instruction::Call(0));
                }
            }
        }
        ir::Value::Dynamic(..) => {
            unreachable!("dynamic only exists inside module access");
        }
        ir::Value::Assert(assert) => {
            let failed = context.make_label("assertion_error");
            write_expression(context, &assert.assertion);
            context
                .instruction(Instruction::Copy)
                .typecheck("boolean")
                .cond_jump(&failed)
                .bubble(|context| {
                    context.label(failed);
                    write_expression(context, &assert.message);
                    context
                        .atom("AssertionError")
                        .instruction(Instruction::Construct)
                        .instruction(Instruction::Panic);
                });
        }
        ir::Value::End => {
            context.instruction(Instruction::Fizzle);
        }
    }
}

fn write_iterator(
    context: &mut Context,
    iterator: &Iterator,
    r#continue: Option<Offset>,
    r#break: Option<Offset>,
) {
    let on_fail = context.make_label("iter_out");
    context
        .closure(|context| {
            context.declare_variables(iterator.query.bindings());
            write_query_state(context, &iterator.query);
            let state = context.intermediate();
            context
                .closure(|context| {
                    context.instruction(Instruction::LoadLocal(state));
                    write_query(context, &iterator.query, &on_fail);
                    context.instruction(Instruction::SetLocal(state));

                    if let Some(r#break) = r#break {
                        context.scope.push_break(r#break);
                    }
                    if let Some(r#continue) = r#continue {
                        context.scope.push_continue(r#continue);
                    }
                    match &iterator.value.value {
                        ir::Value::Mapping(mapping) => {
                            write_expression(context, &mapping.0);
                            context.intermediate();
                            write_expression(context, &mapping.1);
                            context.end_intermediate().instruction(Instruction::Cons);
                        }
                        other => write_evaluation(context, other),
                    }
                    if r#break.is_some() {
                        context.scope.pop_break();
                    }
                    if r#continue.is_some() {
                        context.scope.pop_continue();
                    }
                    context
                        .atom("next")
                        .instruction(Instruction::Construct)
                        .instruction(Instruction::Return)
                        .label(on_fail)
                        .atom("done")
                        .instruction(Instruction::Return);
                })
                .instruction(Instruction::Return)
                .end_intermediate(); // state
            context.undeclare_variables(iterator.query.bindings(), false);
        })
        .instruction(Instruction::Call(0));
}
