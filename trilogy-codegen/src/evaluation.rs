use crate::preamble::{END, RETURN};
use crate::{prelude::*, ASSIGN};
use trilogy_ir::ir;
use trilogy_ir::visitor::HasBindings;
use trilogy_vm::{Array, Instruction, Record, Set, Value};

#[inline(always)]
pub(crate) fn write_expression(context: &mut Context, expr: &ir::Expression) {
    write_evaluation(context, &expr.value)
}

pub(crate) fn write_evaluation(context: &mut Context, value: &ir::Value) {
    match &value {
        ir::Value::Query(..) => unreachable!("query may not appear in an evaluation"),
        ir::Value::Pack(..) => panic!("pack may not appear in an evaluation"),
        ir::Value::Mapping(..) => panic!("mapping may not appear in an evaluation"),
        ir::Value::Conjunction(..) => panic!("conjunction may not appear in an evaluation"),
        ir::Value::Disjunction(..) => panic!("disjunction may not appear in an evaluation"),
        ir::Value::Wildcard => panic!("wildcard may not appear in an evaluation"),
        ir::Value::Builtin(builtin) if is_referenceable_operator(*builtin) => {
            write_operator_reference(context, *builtin);
        }
        ir::Value::Builtin(builtin) => panic!("{builtin:?} is not a referenceable builtin"),
        ir::Value::Sequence(seq) => {
            context.sequence(seq);
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
            context.constant(());
        }
        ir::Value::Atom(value) => {
            context.atom(value);
        }
        ir::Value::Iterator(iter) => {
            context.iterator(iter, None, None).constant(());
        }
        ir::Value::While(stmt) => {
            context.r#while(&stmt.condition.value, &stmt.body.value);
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
                    if element.is_spread {
                        write_expression(context, &element.expression);
                        context.typecheck("record").instruction(Instruction::Glue);
                    } else if let ir::Value::Mapping(mapping) = &element.expression.value {
                        write_expression(context, &mapping.0);
                        context.scope.intermediate();
                        write_expression(context, &mapping.1);
                        context.scope.end_intermediate();
                        context.instruction(Instruction::Assign);
                    } else {
                        panic!("record values must be mappings")
                    }
                }
                context.scope.end_intermediate();
            }
            (None, ir::Value::Builtin(ir::Builtin::Record), ir::Value::Iterator(iter)) => {
                context.comprehension(
                    |context| {
                        context
                            .typecheck("tuple")
                            .instruction(Instruction::Uncons)
                            .instruction(Instruction::Assign);
                    },
                    |context| {
                        context
                            .iterator(iter, None, None)
                            .constant(Record::default());
                    },
                );
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
            (None, ir::Value::Builtin(ir::Builtin::Set), ir::Value::Iterator(iter)) => {
                context.comprehension(
                    |context| {
                        context.instruction(Instruction::Insert);
                    },
                    |context| {
                        context.iterator(iter, None, None).constant(Set::default());
                    },
                );
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
            (None, ir::Value::Builtin(ir::Builtin::Array), ir::Value::Iterator(iter)) => {
                context.comprehension(
                    |context| {
                        context.instruction(Instruction::Insert);
                    },
                    |context| {
                        context
                            .iterator(iter, None, None)
                            .constant(Array::default());
                    },
                );
            }
            (None, ir::Value::Builtin(ir::Builtin::Array), ..) => {
                unreachable!("array is applied to pack or iterator");
            }
            (None, ir::Value::Builtin(ir::Builtin::For), ir::Value::Iterator(iter)) => {
                let continued = context.make_label("for_next");
                let broke = context.make_label("for_broke");

                let did_match = context.constant(false).intermediate();
                let r#break = context
                    .continuation_fn(|c| {
                        c.jump(&broke);
                    })
                    .intermediate();
                context.declare_variables(iter.query.bindings());
                write_query_state(context, &iter.query);
                context
                    .repeat(|context, exit| {
                        write_query(context, &iter.query, exit);
                        context
                            // Mark down that this loop did get a match
                            .constant(true)
                            .instruction(Instruction::SetLocal(did_match))
                            .intermediate(); // query state

                        let r#continue = context
                            .continuation_fn(|c| {
                                c.jump(&continued);
                            })
                            .intermediate();
                        context.push_continue(r#continue).push_break(r#break);
                        write_expression(context, &iter.value);
                        context
                            .pop_break()
                            .pop_continue()
                            // Discard the now invalid "continue" keyword (second from top)
                            .instruction(Instruction::Swap)
                            .instruction(Instruction::Pop)
                            .end_intermediate()
                            .label(&continued)
                            // Then discard the body value... for now. Someday will find a use for it.
                            .instruction(Instruction::Pop)
                            .end_intermediate(); // state (no longer intermediate)
                    })
                    // Discard query state
                    .instruction(Instruction::Pop)
                    .undeclare_variables(iter.query.bindings(), true);
                context
                    .label(&broke)
                    // Remove the break (or break value)
                    .instruction(Instruction::Pop)
                    .end_intermediate()
                    .end_intermediate(); // did match (no longer intermediate)
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
                match &application.argument.value {
                    ir::Value::Pack(pack) => {
                        let arity = pack
                            .len()
                            .expect("procedures may not have spread arguments");
                        for element in &pack.values {
                            write_expression(context, &element.expression);
                        }
                        context.call_procedure(arity);
                    }
                    _ => {
                        write_expression(context, &application.argument);
                        context.call_function();
                    }
                };
                context.scope.end_intermediate();
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
                |context, params| {
                    for handler in &handled.handlers {
                        context
                            .case(|context, next| {
                                context.declare_variables(handler.pattern.bindings());
                                context.instruction(Instruction::LoadLocal(params.effect));
                                write_pattern_match(context, &handler.pattern, next);
                                write_expression(context, &handler.guard);
                                context
                                    .typecheck("boolean")
                                    .cond_jump(next)
                                    .push_cancel(params.cancel)
                                    .push_resume(params.resume);
                                write_expression(context, &handler.body);
                                context
                                    .pop_cancel()
                                    .pop_resume()
                                    // At the end of an effect handler, everything just ends.
                                    //
                                    // The effect handler should have used cancel or resume appropriately
                                    // if "just end" was not the intention.
                                    .instruction(Instruction::Fizzle);
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
