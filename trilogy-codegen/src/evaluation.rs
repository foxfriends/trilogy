use crate::preamble::{END, RETURN};
use crate::prelude::*;
use trilogy_ir::ir;
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
        ir::Value::Builtin(ir::Builtin::Resume) => todo!(),
        ir::Value::Builtin(ir::Builtin::Cancel) => todo!(),
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
                context.write_instruction(Instruction::Pop);
            }
        }
        ir::Value::Assignment(assignment) => match &assignment.lhs.value {
            ir::Value::Reference(var) => {
                write_expression(context, &assignment.rhs);
                match context.scope.lookup(&var.id) {
                    Some(Binding::Variable(index)) => {
                        context
                            .write_instruction(Instruction::Copy)
                            .write_instruction(Instruction::SetLocal(index));
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
            let begin = context.labeler.unique_hint("while");
            let setup = context.labeler.unique_hint("while_setup");
            let cond_fail = context.labeler.unique_hint("while_exit");
            let end = context.labeler.unique_hint("while_end");
            let continuation = context.labeler.unique_hint("while_cont");
            let r#continue = context.scope.push_continue();
            let r#break = context.scope.push_break();
            context
                .write_instruction(Instruction::Const(Value::Unit))
                .write_instruction(Instruction::Const(Value::Unit))
                .shift(&continuation)
                .write_label(begin.to_owned());
            write_expression(context, &stmt.condition);
            context.cond_jump(&cond_fail);
            write_expression(context, &stmt.body);
            context
                .write_instruction(Instruction::LoadLocal(r#continue))
                .write_instruction(Instruction::Become(0))
                .write_label(cond_fail)
                .write_instruction(Instruction::LoadLocal(r#break))
                .write_instruction(Instruction::Become(0))
                .write_label(continuation)
                .write_instruction(Instruction::SetLocal(r#continue))
                .shift(&setup)
                .write_instruction(Instruction::Pop)
                .write_instruction(Instruction::Pop)
                .jump(&end)
                .write_label(setup)
                .write_instruction(Instruction::SetLocal(r#break))
                .jump(&begin)
                .write_label(end);
            context.scope.pop_break();
            context.scope.pop_continue();
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
                            context.write_instruction(Instruction::Glue);
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
                            context.write_instruction(Instruction::Glue);
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
                            context.write_instruction(Instruction::Glue);
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
            context.write_label(when_false);
            write_expression(context, &cond.when_false);
            context.write_label(end);
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
                context.write_label(cleanup);
                for _ in 0..vars {
                    context.write_instruction(Instruction::Pop);
                }
            }
            context.scope.end_intermediate();
            context.write_label(end);
        }
        ir::Value::Fn(closure) => {
            let end = context.labeler.unique_hint("end_fn");
            let params = context.scope.closure(closure.parameters.len());
            for i in 0..closure.parameters.len() {
                context.close(if i == 0 { &end } else { RETURN });
            }
            for (i, parameter) in closure.parameters.iter().enumerate() {
                context.declare_variables(parameter.bindings());
                context.write_instruction(Instruction::LoadLocal(params + i));
                write_pattern_match(context, parameter, END);
            }
            write_expression(context, &closure.body);
            for parameter in closure.parameters.iter().rev() {
                context.undeclare_variables(parameter.bindings(), false);
                context.scope.unclosure(1)
            }

            context.write_label(end);
        }
        ir::Value::Do(closure) => {
            let end = context.labeler.unique_hint("end_do");
            let param_start = context.scope.closure(closure.parameters.len());
            context.close(&end);
            for (offset, parameter) in closure.parameters.iter().enumerate() {
                context.declare_variables(parameter.bindings());
                context.write_instruction(Instruction::LoadLocal(param_start + offset));
                write_pattern_match(context, parameter, END);
            }
            write_expression(context, &closure.body);
            context
                .write_instruction(Instruction::Const(Value::Unit))
                .write_instruction(Instruction::Return)
                .write_label(end);
            for parameter in closure.parameters.iter().rev() {
                context.undeclare_variables(parameter.bindings(), false);
            }
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
