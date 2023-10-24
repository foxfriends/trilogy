use crate::preamble::{END, RETURN};
use crate::prelude::*;
use trilogy_ir::ir;
use trilogy_ir::visitor::HasBindings;
use trilogy_vm::{Instruction, Value};

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
            let Some(mut expr) = seq.next() else { return };
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
                    write_evaluation(context, collection);
                    context.scope.intermediate();
                    write_evaluation(context, key);
                    context.scope.intermediate();
                    write_expression(context, &assignment.rhs);
                    context.scope.end_intermediate();
                    context.scope.end_intermediate();
                    context.instruction(Instruction::Assign);
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
            context.instruction(Instruction::Const(value.value().clone().into()));
        }
        ir::Value::Character(value) => {
            context.instruction(Instruction::Const((*value).into()));
        }
        ir::Value::String(value) => {
            context.instruction(Instruction::Const(value.into()));
        }
        ir::Value::Bits(value) => {
            context.instruction(Instruction::Const(value.value().clone().into()));
        }
        ir::Value::Boolean(value) => {
            context.instruction(Instruction::Const((*value).into()));
        }
        ir::Value::Unit => {
            context.instruction(Instruction::Const(Value::Unit));
        }
        ir::Value::Conjunction(..) => unreachable!("Conjunction cannot appear in an evaluation"),
        ir::Value::Disjunction(..) => unreachable!("Disjunction cannot appear in an evaluation"),
        ir::Value::Wildcard => unreachable!("Wildcard cannot appear in an evaluation"),
        ir::Value::Atom(value) => {
            let atom = context.atom(value);
            context.instruction(Instruction::Const(atom.into()));
        }
        ir::Value::Query(..) => unreachable!("Query cannot appear in an evaluation"),
        ir::Value::Iterator(iterator) => {
            let construct = context.labeler.unique_hint("iter");
            let on_fail = context.labeler.unique_hint("iter_out");
            let end = context.labeler.unique_hint("iter_end");
            context.close(&construct);
            context.declare_variables(iterator.query.bindings());
            write_query_state(context, &iterator.query);
            let state = context.scope.intermediate();
            context.close(&end);
            context.instruction(Instruction::LoadLocal(state));
            write_query(context, &iterator.query, &on_fail);
            context.instruction(Instruction::SetLocal(state));
            match &iterator.value.value {
                ir::Value::Mapping(mapping) => {
                    write_expression(context, &mapping.0);
                    context.scope.intermediate();
                    write_expression(context, &mapping.1);
                    context.scope.end_intermediate();
                    context.instruction(Instruction::Cons);
                }
                other => write_evaluation(context, other),
            }
            let next = context.atom("next");
            let done = context.atom("done");
            context
                .instruction(Instruction::Const(next.into()))
                .instruction(Instruction::Construct)
                .instruction(Instruction::Return)
                .label(on_fail)
                .instruction(Instruction::Const(done.into()))
                .label(end) // end is just here to reuse a return instead of printing two in a row
                .instruction(Instruction::Return)
                .label(construct)
                .instruction(Instruction::Call(0));
            context.scope.end_intermediate();
            context.undeclare_variables(iterator.query.bindings(), false);
        }
        ir::Value::While(stmt) => {
            let begin = context.labeler.unique_hint("while");
            let setup = context.labeler.unique_hint("while_setup");
            let cond_fail = context.labeler.unique_hint("while_exit");
            let end = context.labeler.unique_hint("while_end");
            let continuation = context.labeler.unique_hint("while_cont");
            let r#continue = context.scope.push_continue();
            let r#break = context.scope.push_break();
            context
                .instruction(Instruction::Const(Value::Unit))
                .instruction(Instruction::Const(Value::Unit))
                .shift(&continuation)
                .label(begin.to_owned());
            write_expression(context, &stmt.condition);
            context.cond_jump(&cond_fail);
            write_expression(context, &stmt.body);
            context
                .instruction(Instruction::LoadLocal(r#continue))
                .instruction(Instruction::Become(0))
                .label(cond_fail)
                .instruction(Instruction::LoadLocal(r#break))
                .instruction(Instruction::Become(0))
                .label(continuation)
                .instruction(Instruction::SetLocal(r#continue))
                .shift(&setup)
                .instruction(Instruction::Pop)
                .instruction(Instruction::Pop)
                .jump(&end)
                .label(setup)
                .instruction(Instruction::SetLocal(r#break))
                .jump(&begin)
                .label(end);
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
                let atom = context.atom((**ident).as_ref());
                context
                    .instruction(Instruction::Const(atom.into()))
                    .instruction(Instruction::Call(1));
            }
            (None, ir::Value::Builtin(builtin), arg) if is_operator(*builtin) => {
                write_unary_operation(context, arg, *builtin);
            }
            (Some(ir::Value::Builtin(builtin)), lhs, rhs) if is_operator(*builtin) => {
                write_binary_operation(context, lhs, rhs, *builtin);
            }
            (None, ir::Value::Builtin(ir::Builtin::Record), arg) => {
                context.instruction(Instruction::Const(Value::Record(Default::default())));
                let record = context.scope.intermediate();
                match arg {
                    ir::Value::Pack(pack) => {
                        for element in &pack.values {
                            write_expression(context, &element.expression);
                            if element.is_spread {
                                context.instruction(Instruction::Glue);
                            } else {
                                context.instruction(Instruction::Assign);
                            }
                        }
                    }
                    ir::Value::Iterator(..) => {
                        write_evaluation(context, arg);
                        context.scope.intermediate();
                        let loop_begin = context.labeler.unique_hint("record_collect");
                        let loop_exit = context.labeler.unique_hint("record_collect_end");
                        let next = context.atom("next");
                        let done = context.atom("done");
                        let struct_atom = context.atom("struct");
                        context
                            .label(loop_begin.clone())
                            .instruction(Instruction::Copy)
                            .instruction(Instruction::Call(0))
                            .instruction(Instruction::Copy)
                            .instruction(Instruction::Const(done.into()))
                            .instruction(Instruction::ValNeq)
                            .cond_jump(&loop_exit)
                            .instruction(Instruction::Copy)
                            .instruction(Instruction::TypeOf)
                            .instruction(Instruction::Const(struct_atom.into()))
                            .instruction(Instruction::ValEq)
                            .cond_jump(END)
                            .instruction(Instruction::Destruct)
                            .instruction(Instruction::Const(next.into()))
                            .instruction(Instruction::ValEq)
                            .cond_jump(END)
                            .instruction(Instruction::LoadLocal(record))
                            .instruction(Instruction::Swap)
                            .instruction(Instruction::Uncons)
                            .instruction(Instruction::Assign)
                            .instruction(Instruction::Pop)
                            .jump(&loop_begin)
                            .label(loop_exit)
                            .instruction(Instruction::Pop)
                            .instruction(Instruction::Pop);
                        context.scope.end_intermediate();
                    }
                    _ => panic!("record literal must have pack or iterator"),
                }
                context.scope.end_intermediate();
            }
            (None, ir::Value::Builtin(ir::Builtin::Set), arg) => {
                context.instruction(Instruction::Const(Value::Set(Default::default())));
                let set = context.scope.intermediate();
                match arg {
                    ir::Value::Pack(pack) => {
                        for element in &pack.values {
                            write_expression(context, &element.expression);
                            if element.is_spread {
                                context.instruction(Instruction::Glue);
                            } else {
                                context.instruction(Instruction::Insert);
                            }
                        }
                    }
                    ir::Value::Iterator(..) => {
                        write_evaluation(context, arg);
                        context.scope.intermediate();
                        let loop_begin = context.labeler.unique_hint("set_collect");
                        let loop_exit = context.labeler.unique_hint("set_collect_end");
                        let next = context.atom("next");
                        let done = context.atom("done");
                        let struct_atom = context.atom("struct");
                        context
                            .label(loop_begin.clone())
                            .instruction(Instruction::Copy)
                            .instruction(Instruction::Call(0))
                            .instruction(Instruction::Copy)
                            .instruction(Instruction::Const(done.into()))
                            .instruction(Instruction::ValNeq)
                            .cond_jump(&loop_exit)
                            .instruction(Instruction::Copy)
                            .instruction(Instruction::TypeOf)
                            .instruction(Instruction::Const(struct_atom.into()))
                            .instruction(Instruction::ValEq)
                            .cond_jump(END)
                            .instruction(Instruction::Destruct)
                            .instruction(Instruction::Const(next.into()))
                            .instruction(Instruction::ValEq)
                            .cond_jump(END)
                            .instruction(Instruction::LoadLocal(set))
                            .instruction(Instruction::Swap)
                            .instruction(Instruction::Insert)
                            .instruction(Instruction::Pop)
                            .jump(&loop_begin)
                            .label(loop_exit)
                            .instruction(Instruction::Pop)
                            .instruction(Instruction::Pop);
                        context.scope.end_intermediate();
                    }
                    _ => panic!("set literal must have pack or iterator"),
                }
                context.scope.end_intermediate();
            }
            (None, ir::Value::Builtin(ir::Builtin::Array), arg) => {
                context.instruction(Instruction::Const(Value::Array(Default::default())));
                let array = context.scope.intermediate();
                match arg {
                    ir::Value::Pack(pack) => {
                        for element in &pack.values {
                            write_expression(context, &element.expression);
                            if element.is_spread {
                                context.instruction(Instruction::Glue);
                            } else {
                                context.instruction(Instruction::Insert);
                            }
                        }
                    }
                    ir::Value::Iterator(..) => {
                        write_evaluation(context, arg);
                        context.scope.intermediate();
                        let loop_begin = context.labeler.unique_hint("array_collect");
                        let loop_exit = context.labeler.unique_hint("array_collect_end");
                        let next = context.atom("next");
                        let done = context.atom("done");
                        let struct_atom = context.atom("struct");
                        context
                            .label(loop_begin.clone())
                            .instruction(Instruction::Copy)
                            .instruction(Instruction::Call(0))
                            .instruction(Instruction::Copy)
                            .instruction(Instruction::Const(done.into()))
                            .instruction(Instruction::ValNeq)
                            .cond_jump(&loop_exit)
                            .instruction(Instruction::Copy)
                            .instruction(Instruction::TypeOf)
                            .instruction(Instruction::Const(struct_atom.into()))
                            .instruction(Instruction::ValEq)
                            .cond_jump(END)
                            .instruction(Instruction::Destruct)
                            .instruction(Instruction::Const(next.into()))
                            .instruction(Instruction::ValEq)
                            .cond_jump(END)
                            .instruction(Instruction::LoadLocal(array))
                            .instruction(Instruction::Swap)
                            .instruction(Instruction::Insert)
                            .instruction(Instruction::Pop)
                            .jump(&loop_begin)
                            .label(loop_exit)
                            .instruction(Instruction::Pop)
                            .instruction(Instruction::Pop);
                        context.scope.end_intermediate();
                    }
                    _ => panic!("array literal must have pack or iterator"),
                }
                context.scope.end_intermediate();
            }
            (None, ir::Value::Builtin(ir::Builtin::For), value) => {
                context.instruction(Instruction::Const(false.into()));
                let eval_to = context.scope.intermediate();
                write_evaluation(context, value);
                let loop_begin = context.labeler.unique_hint("for");
                let loop_exit = context.labeler.unique_hint("for_end");
                let next = context.atom("next");
                let done = context.atom("done");
                let struct_atom = context.atom("struct");
                context
                    .label(loop_begin.clone())
                    .instruction(Instruction::Copy)
                    .instruction(Instruction::Call(0))
                    .instruction(Instruction::Copy)
                    .instruction(Instruction::Const(done.into()))
                    .instruction(Instruction::ValNeq)
                    .cond_jump(&loop_exit)
                    .instruction(Instruction::Copy)
                    .instruction(Instruction::TypeOf)
                    .instruction(Instruction::Const(struct_atom.into()))
                    .instruction(Instruction::ValEq)
                    .cond_jump(END)
                    .instruction(Instruction::Destruct)
                    .instruction(Instruction::Const(next.into()))
                    .instruction(Instruction::ValEq)
                    .cond_jump(END)
                    .instruction(Instruction::Const(true.into()))
                    .instruction(Instruction::SetLocal(eval_to))
                    .instruction(Instruction::Pop)
                    .jump(&loop_begin)
                    .label(loop_exit)
                    .instruction(Instruction::Pop)
                    .instruction(Instruction::Pop);
                context.scope.end_intermediate();
            }
            (None, ir::Value::Builtin(ir::Builtin::Is), ir::Value::Query(query)) => {
                let is_fail = context.labeler.unique_hint("is_fail");
                let is_end = context.labeler.unique_hint("is_end");
                let var_count = context.declare_variables(query.bindings());
                write_query_state(context, query);
                write_query(context, query, &is_fail);
                context
                    .instruction(Instruction::Const(true.into()))
                    .instruction(Instruction::Slide(var_count as u32 + 1))
                    .jump(&is_end)
                    .label(&is_fail)
                    .instruction(Instruction::Const(false.into()))
                    .instruction(Instruction::Slide(var_count as u32 + 1))
                    .label(is_end);
                for _ in 0..=var_count {
                    // One extra POP to discard the query state
                    context.instruction(Instruction::Pop);
                }
            }
            _ => {
                write_expression(context, &application.function);
                context.scope.intermediate();
                write_expression(context, &application.argument);
                context.scope.end_intermediate();
                let arity = match &application.argument.value {
                    ir::Value::Pack(pack) => pack
                        .len()
                        .expect("procedures may not have spread arguments")
                        as u32,
                    _ => 1,
                };
                context.instruction(Instruction::Call(arity));
            }
        },
        ir::Value::Let(decl) if decl.query.is_once() => {
            let reenter = context.labeler.unique_hint("let");
            context.declare_variables(decl.query.bindings());
            write_query_state(context, &decl.query);
            context.label(reenter.clone());
            write_query(context, &decl.query, END);
            context.scope.intermediate(); // after running the query, its state is an intermediate
            write_expression(context, &decl.body);

            // TODO: would be really nice to move this pop (of the query state) one
            // line up, but the shared stack thing with closures makes it not work
            context.instruction(Instruction::Pop);
            context.scope.end_intermediate(); // query state
            context.undeclare_variables(decl.query.bindings(), true);
        }
        ir::Value::Let(decl) => {
            let reenter = context.labeler.unique_hint("let");

            context.declare_variables(decl.query.bindings());
            write_query_state(context, &decl.query);
            context.label(reenter.clone());
            write_query(context, &decl.query, END);
            context.scope.intermediate();
            context
                .instruction(Instruction::Const(Value::Bool(true)))
                .instruction(Instruction::Const(Value::Bool(false)))
                .instruction(Instruction::Branch)
                .cond_jump(&reenter);
            write_expression(context, &decl.body);
            context.instruction(Instruction::Pop);
            context.scope.end_intermediate();
            context.undeclare_variables(decl.query.bindings(), true);
        }
        ir::Value::IfElse(cond) => {
            let when_false = context.labeler.unique_hint("else");
            write_expression(context, &cond.condition);
            context.cond_jump(&when_false);
            write_expression(context, &cond.when_true);
            let end = context.labeler.unique_hint("end_if");
            context.jump(&end);
            context.label(when_false);
            write_expression(context, &cond.when_false);
            context.label(end);
        }
        ir::Value::Match(cond) => {
            write_expression(context, &cond.expression);
            let val = context.scope.intermediate();
            let end = context.labeler.unique_hint("match_end");
            for case in &cond.cases {
                let cleanup = context.labeler.unique_hint("case_cleanup");
                let vars = context.declare_variables(case.pattern.bindings());
                context.instruction(Instruction::LoadLocal(val));
                write_pattern_match(context, &case.pattern, &cleanup);
                write_expression(context, &case.guard);
                context.cond_jump(&cleanup);
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
            let end = context.labeler.unique_hint("end_fn");
            let params = context.scope.closure(closure.parameters.len());
            for i in 0..closure.parameters.len() {
                context.close(if i == 0 { &end } else { RETURN });
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
            let end = context.labeler.unique_hint("end_do");
            let param_start = context.scope.closure(closure.parameters.len());
            context.close(&end);
            for (offset, parameter) in closure.parameters.iter().enumerate() {
                context.declare_variables(parameter.bindings());
                context.instruction(Instruction::LoadLocal(param_start + offset as u32));
                write_pattern_match(context, parameter, END);
            }
            write_expression(context, &closure.body);
            context
                .instruction(Instruction::Const(Value::Unit))
                .instruction(Instruction::Return)
                .label(end);
            for parameter in closure.parameters.iter().rev() {
                context.undeclare_variables(parameter.bindings(), false);
            }
            context.scope.unclosure(closure.parameters.len());
        }
        ir::Value::Handled(handled) => {
            let when = context.labeler.unique_hint("when");
            let end = context.labeler.unique_hint("with_end");
            let body = context.labeler.unique_hint("with_body");

            // Register 0 holds the current effect handler, which corresponds to the `yield` keyword.
            // To register a new one we must first store the parent handler.
            context.instruction(Instruction::LoadRegister(0));
            let stored_yield = context.scope.intermediate();

            // Second on the stack is the cancel continuation.
            context.shift(&when).jump(&end);
            context.scope.push_cancel();

            // The new yield is created next, but it's not kept on stack.
            context.label(when).shift(&body);

            // That new yield will be called with the effect and the resume continuation.
            let effect = context.scope.intermediate();
            context.scope.push_resume();

            // Immediately restore the context of the parent `yield`, as a handler may use it.
            // The `yield` used to arrive here isn't needed though, as the `yield` expression
            // itself takes care of saving that to restore it when resumed.
            context
                .instruction(Instruction::LoadLocal(stored_yield))
                .instruction(Instruction::SetRegister(0));
            for handler in &handled.handlers {
                let next = context.labeler.unique_hint("when_next");
                context.declare_variables(handler.pattern.bindings());
                context.instruction(Instruction::LoadLocal(effect));
                write_pattern_match(context, &handler.pattern, &next);
                write_expression(context, &handler.guard);
                context.cond_jump(&next);
                write_expression(context, &handler.body);
                context
                    .instruction(Instruction::Fizzle)
                    .label(next)
                    .undeclare_variables(handler.pattern.bindings(), true);
            }
            context.instruction(Instruction::Fizzle);
            context.scope.pop_resume();
            context.scope.end_intermediate(); // effect

            // The body of the `when` statement involves saving the `yield` that was just created,
            // running the expression, and then cleaning up.
            context.label(body).instruction(Instruction::SetRegister(0));
            write_expression(context, &handled.expression);
            context
                // When the expression finishes evaluation, we reset from any shifted continuations.
                // If there were none, then `reset` should be noop, in which case we have to remove
                // the cancel from the stack.
                .instruction(Instruction::Reset)
                .instruction(Instruction::Swap)
                .instruction(Instruction::Pop);
            context.scope.pop_cancel();
            context
                .label(end)
                // Once we're out of the handler (due to runoff or cancel), reset the state of the
                // `yield` register and finally done!
                .instruction(Instruction::Swap)
                .instruction(Instruction::SetRegister(0));
            context.scope.end_intermediate(); // stored yield
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
                    context.write_procedure_reference(label.to_owned());
                }
                Binding::Context(label) => {
                    context
                        .write_procedure_reference(label.to_owned())
                        .instruction(Instruction::Call(0));
                }
                Binding::Chunk(path) => {
                    context.instruction(Instruction::Chunk(path.into()));
                }
            }
        }
        ir::Value::Dynamic(..) => {
            unreachable!("dynamic only exists inside module access");
        }
        ir::Value::Assert(..) => todo!(),
        ir::Value::End => {
            context.instruction(Instruction::Fizzle);
        }
    }
}

fn write_unary_operation(context: &mut Context, value: &ir::Value, builtin: ir::Builtin) {
    write_evaluation(context, value);
    write_operator(context, builtin);
}

fn write_binary_operation(
    context: &mut Context,
    lhs: &ir::Value,
    rhs: &ir::Value,
    builtin: ir::Builtin,
) {
    write_evaluation(context, lhs);
    context.scope.intermediate();
    write_evaluation(context, rhs);
    context.scope.end_intermediate();
    write_operator(context, builtin);
}
