use crate::prelude::*;
use trilogy_ir::ir;
use trilogy_vm::{Instruction, Value};

#[inline(always)]
pub(crate) fn write_expression(context: &mut Context, expr: &ir::Expression) {
    write_evaluation(context, &expr.value)
}

pub(crate) fn write_evaluation(context: &mut Context, value: &ir::Value) {
    match &value {
        ir::Value::Builtin(..) => todo!("{value:?}"),
        ir::Value::Pack(pack) => {
            for element in &pack.values {
                if element.is_spread {
                    todo!()
                } else {
                    write_expression(context, &element.expression);
                }
            }
        }
        ir::Value::Sequence(seq) => {
            for expr in seq {
                write_expression(context, expr);
            }
        }
        ir::Value::Assignment(assignment) => match &assignment.lhs.value {
            ir::Value::Reference(var) => {
                write_expression(context, &assignment.rhs);
                match context.scope.lookup(&var.id) {
                    Some(Binding::Variable(index)) => {
                        context.write_instruction(Instruction::SetLocal(index));
                    }
                    _ => unreachable!("Only variables can be assigned to"),
                }
            }
            ir::Value::Application(application) => match unapply_2(application) {
                (Some(ir::Value::Builtin(ir::Builtin::Access)), collection, key) => {
                    write_evaluation(context, collection);
                    write_evaluation(context, key);
                    write_expression(context, &assignment.rhs);
                    context.write_instruction(Instruction::Assign);
                }
                _ => unreachable!("LValue applications must be access"),
            },
            _ => unreachable!("LValues must be reference or application"),
        },
        ir::Value::Mapping(value) => {
            write_expression(context, &value.0);
            write_expression(context, &value.1);
        }
        ir::Value::Number(value) => {
            context.write_instruction(Instruction::Const(value.value().clone().into()));
        }
        ir::Value::Character(value) => {
            context.write_instruction(Instruction::Const((*value).into()));
        }
        ir::Value::String(value) => {
            context.write_instruction(Instruction::Const(value.into()));
        }
        ir::Value::Bits(value) => {
            context.write_instruction(Instruction::Const(value.value().clone().into()));
        }
        ir::Value::Boolean(value) => {
            context.write_instruction(Instruction::Const((*value).into()));
        }
        ir::Value::Unit => {
            context.write_instruction(Instruction::Const(Value::Unit));
        }
        ir::Value::Conjunction(..) => unreachable!("Conjunction cannot appear in an evaluation"),
        ir::Value::Disjunction(..) => unreachable!("Disjunction cannot appear in an evaluation"),
        ir::Value::Wildcard => unreachable!("Wildcard cannot appear in an evaluation"),
        ir::Value::Atom(value) => {
            let atom = context.atom(value);
            context.write_instruction(Instruction::Const(atom.into()));
        }
        ir::Value::Query(..) => unreachable!("Query cannot appear in an evaluation"),
        ir::Value::Iterator(..) => todo!(),
        ir::Value::While(stmt) => {
            // TODO: support continue/break
            let start = context.labeler.unique_hint("while");
            let end = context.labeler.unique_hint("end_while");
            context.write_label(start.clone()).unwrap();
            write_expression(context, &stmt.condition);
            context.cond_jump(&end);
            write_expression(context, &stmt.body);
            context.jump(&start);
            context.write_label(end).unwrap();
        }
        ir::Value::Application(application) => match unapply_2(application) {
            (None, ir::Value::Builtin(builtin), arg) if is_operator(*builtin) => {
                write_unary_operation(context, arg, *builtin);
            }
            (Some(ir::Value::Builtin(builtin)), lhs, rhs) if is_operator(*builtin) => {
                write_binary_operation(context, lhs, rhs, *builtin);
            }
            (None, ir::Value::Builtin(ir::Builtin::Record), arg) => match arg {
                ir::Value::Pack(pack) => {
                    context
                        .write_instruction(Instruction::Const(Value::Record(Default::default())));
                    for element in &pack.values {
                        write_expression(context, &element.expression);
                        if element.is_spread {
                            let spread = context.labeler.unique_hint("spread");
                            let end_spread = context.labeler.unique_hint("end_spread");
                            context
                                .write_instruction(Instruction::Entries)
                                .write_instruction(Instruction::Const(0.into()))
                                .write_instruction(Instruction::Swap)
                                .write_label(spread.clone())
                                .unwrap()
                                .write_instruction(Instruction::Copy)
                                .write_instruction(Instruction::Length)
                                .write_instruction(Instruction::LoadRegister(2))
                                .write_instruction(Instruction::ValNeq)
                                .cond_jump(&end_spread)
                                .write_instruction(Instruction::Copy)
                                .write_instruction(Instruction::LoadRegister(2))
                                .write_instruction(Instruction::Access)
                                .write_instruction(Instruction::LoadRegister(3))
                                .write_instruction(Instruction::Swap)
                                .write_instruction(Instruction::Uncons)
                                .write_instruction(Instruction::Assign)
                                .write_instruction(Instruction::Pop)
                                .write_instruction(Instruction::Swap)
                                .write_instruction(Instruction::Const(1.into()))
                                .write_instruction(Instruction::Add)
                                .write_instruction(Instruction::Swap)
                                .jump(&spread)
                                .write_label(end_spread)
                                .unwrap()
                                .write_instruction(Instruction::Pop)
                                .write_instruction(Instruction::Pop);
                        } else {
                            context.write_instruction(Instruction::Assign);
                        }
                    }
                }
                _ => todo!("{arg:?}"),
            },
            (None, ir::Value::Builtin(ir::Builtin::Set), arg) => match arg {
                ir::Value::Pack(pack) => {
                    context.write_instruction(Instruction::Const(Value::Set(Default::default())));
                    for element in &pack.values {
                        write_expression(context, &element.expression);
                        if element.is_spread {
                            let spread = context.labeler.unique_hint("spread");
                            let end_spread = context.labeler.unique_hint("end_spread");
                            context
                                .write_instruction(Instruction::Entries)
                                .write_instruction(Instruction::Const(0.into()))
                                .write_instruction(Instruction::Swap)
                                .write_label(spread.clone())
                                .unwrap()
                                .write_instruction(Instruction::Copy)
                                .write_instruction(Instruction::Length)
                                .write_instruction(Instruction::LoadRegister(2))
                                .write_instruction(Instruction::ValNeq)
                                .cond_jump(&end_spread)
                                .write_instruction(Instruction::Copy)
                                .write_instruction(Instruction::LoadRegister(2))
                                .write_instruction(Instruction::Access)
                                .write_instruction(Instruction::LoadRegister(3))
                                .write_instruction(Instruction::Swap)
                                .write_instruction(Instruction::Insert)
                                .write_instruction(Instruction::Pop)
                                .write_instruction(Instruction::Swap)
                                .write_instruction(Instruction::Const(1.into()))
                                .write_instruction(Instruction::Add)
                                .write_instruction(Instruction::Swap)
                                .jump(&spread)
                                .write_label(end_spread)
                                .unwrap()
                                .write_instruction(Instruction::Pop)
                                .write_instruction(Instruction::Pop);
                        } else {
                            context.write_instruction(Instruction::Insert);
                        }
                    }
                }
                _ => todo!("{arg:?}"),
            },
            (None, ir::Value::Builtin(ir::Builtin::Array), arg) => match arg {
                ir::Value::Pack(pack) => {
                    context.write_instruction(Instruction::Const(Value::Array(Default::default())));
                    for element in &pack.values {
                        write_expression(context, &element.expression);
                        if element.is_spread {
                            let spread = context.labeler.unique_hint("spread");
                            let end_spread = context.labeler.unique_hint("end_spread");
                            context
                                .write_instruction(Instruction::Const(0.into()))
                                .write_instruction(Instruction::Swap)
                                .write_label(spread.clone())
                                .unwrap()
                                .write_instruction(Instruction::Copy)
                                .write_instruction(Instruction::Length)
                                .write_instruction(Instruction::LoadRegister(2))
                                .write_instruction(Instruction::ValNeq)
                                .cond_jump(&end_spread)
                                .write_instruction(Instruction::Copy)
                                .write_instruction(Instruction::LoadRegister(2))
                                .write_instruction(Instruction::Access)
                                .write_instruction(Instruction::LoadRegister(3))
                                .write_instruction(Instruction::Swap)
                                .write_instruction(Instruction::Insert)
                                .write_instruction(Instruction::Pop)
                                .write_instruction(Instruction::Swap)
                                .write_instruction(Instruction::Const(1.into()))
                                .write_instruction(Instruction::Add)
                                .write_instruction(Instruction::Swap)
                                .jump(&spread)
                                .write_label(end_spread)
                                .unwrap()
                                .write_instruction(Instruction::Pop)
                                .write_instruction(Instruction::Pop);
                        } else {
                            context.write_instruction(Instruction::Insert);
                        }
                    }
                }
                ir::Value::Iterator(iter) => todo!("{iter:?}"),
                _ => panic!("collections must be pack or iterator"),
            },
            _ => {
                write_expression(context, &application.function);
                write_expression(context, &application.argument);
                let arity = match &application.argument.value {
                    ir::Value::Pack(pack) => pack
                        .len()
                        .expect("procedures may not have spread arguments"),
                    _ => 1,
                };
                context.write_instruction(Instruction::Call(arity));
            }
        },
        ir::Value::Let(decl) => {
            context.declare_variables(decl.query.bindings());
            write_query(context, &decl.query);
            write_expression(context, &decl.body);
            context.undeclare_variables(decl.query.bindings(), true);
        }
        ir::Value::IfElse(cond) => {
            let when_false = context.labeler.unique_hint("else");
            write_expression(context, &cond.condition);
            context.cond_jump(&when_false);
            write_expression(context, &cond.when_true);
            let end = context.labeler.unique_hint("end_if");
            context.jump(&end);
            context.write_label(when_false).unwrap();
            write_expression(context, &cond.when_false);
            context.write_label(end).unwrap();
        }
        ir::Value::Match(cond) => {
            write_expression(context, &cond.expression);
            let end = context.labeler.unique_hint("match_end");
            let val = context.scope.intermediate();
            for case in &cond.cases {
                let cleanup = context.labeler.unique_hint("case_cleanup");
                let vars = context.declare_variables(case.pattern.bindings());
                context.write_instruction(Instruction::LoadLocal(val));
                write_pattern_match(context, &case.pattern, &cleanup);
                write_expression(context, &case.guard);
                context.cond_jump(&cleanup);
                write_expression(context, &case.body);
                context.write_instruction(Instruction::SetLocal(val));
                context.undeclare_variables(case.pattern.bindings(), true);
                context.jump(&end);
                context.write_label(cleanup).unwrap();
                for _ in 0..vars {
                    context.write_instruction(Instruction::Pop);
                }
            }
            context.scope.end_intermediate();
            context.write_label(end).unwrap();
        }
        ir::Value::Fn(closure) => {
            let end = context.labeler.unique_hint("end_fn");
            let reset = context.labeler.unique_hint("mid_fn");
            let on_fail = context.labeler.unique_hint("fn_fail");
            let mut args = vec![];
            for i in 0..closure.parameters.len() {
                args.push(context.scope.closure(1));
                context.shift(if i == 0 { &end } else { &reset });
            }
            for (i, parameter) in closure.parameters.iter().enumerate() {
                context.declare_variables(parameter.bindings());
                context.write_instruction(Instruction::LoadLocal(args[i]));
                write_pattern_match(context, parameter, &on_fail);
            }
            write_expression(context, &closure.body);
            for parameter in closure.parameters.iter().rev() {
                context.undeclare_variables(parameter.bindings(), false);
                context.scope.unclosure(1)
            }

            context
                .write_label(reset)
                .unwrap()
                .write_instruction(Instruction::Reset)
                .write_label(on_fail)
                .unwrap()
                .write_instruction(Instruction::Fizzle)
                .write_label(end)
                .unwrap();
        }
        ir::Value::Do(closure) => {
            let end = context.labeler.unique_hint("end_do");
            context.shift(&end);
            let param_start = context.scope.closure(closure.parameters.len());

            let on_fail = context.labeler.unique_hint("do_fail");
            for (offset, parameter) in closure.parameters.iter().enumerate() {
                context.declare_variables(parameter.bindings());
                context.write_instruction(Instruction::LoadLocal(param_start + offset));
                write_pattern_match(context, parameter, &on_fail);
            }
            write_expression(context, &closure.body);
            for parameter in closure.parameters.iter().rev() {
                context.undeclare_variables(parameter.bindings(), false);
            }

            context
                .write_instruction(Instruction::Const(Value::Unit))
                .write_instruction(Instruction::Reset)
                .write_label(on_fail)
                .unwrap()
                .write_instruction(Instruction::Fizzle)
                .write_label(end)
                .unwrap();

            context.scope.unclosure(closure.parameters.len());
        }
        ir::Value::Handled(..) => todo!("{value:?}"),
        ir::Value::Module(..) => todo!("{value:?}"),
        ir::Value::Reference(ident) => {
            let binding = context
                .scope
                .lookup(&ident.id)
                .expect("unresolved reference should not exist at this point");
            match binding {
                Binding::Variable(offset) => {
                    context.write_instruction(Instruction::LoadLocal(offset));
                }
                Binding::Static(label) => {
                    context.write_procedure_reference(label.to_owned());
                }
            }
        }
        ir::Value::Dynamic(dynamic) => {
            panic!("Dynamic is not actually supposed to happen, but we got {dynamic:?}");
        }
        ir::Value::Assert(..) => todo!(),
        ir::Value::End => {
            context.write_instruction(Instruction::Fizzle);
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
    write_evaluation(context, rhs);
    write_operator(context, builtin);
}
