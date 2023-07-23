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
                        context.write_instruction(Instruction::SetLocal(*index));
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
        ir::Value::Mapping(..) => unreachable!("Mapping cannot appear in an evaluation"),
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
        ir::Value::Query(..) => todo!(),
        ir::Value::Iterator(..) => todo!(),
        ir::Value::While(..) => todo!(),
        ir::Value::Application(application) => match unapply_2(application) {
            (None, ir::Value::Builtin(builtin), arg) if is_operator(*builtin) => {
                write_unary_operation(context, arg, *builtin);
            }
            (Some(ir::Value::Builtin(builtin)), lhs, rhs) if is_operator(*builtin) => {
                write_binary_operation(context, lhs, rhs, *builtin);
            }
            (None, ir::Value::Builtin(ir::Builtin::Record), arg) => match arg {
                ir::Value::Pack(pack) if pack.values.is_empty() => {
                    context
                        .write_instruction(Instruction::Const(Value::Record(Default::default())));
                }
                _ => todo!("{arg:?}"),
            },
            (None, ir::Value::Builtin(ir::Builtin::Set), arg) => match arg {
                ir::Value::Pack(pack) if pack.values.is_empty() => {
                    context.write_instruction(Instruction::Const(Value::Set(Default::default())));
                }
                _ => todo!("{arg:?}"),
            },
            (None, ir::Value::Builtin(ir::Builtin::Array), arg) => match arg {
                ir::Value::Pack(pack) if pack.values.is_empty() => {
                    context.write_instruction(Instruction::Const(Value::Array(Default::default())));
                }
                _ => todo!("{arg:?}"),
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
        ir::Value::Let(..) => todo!("{value:?}"),
        ir::Value::IfElse(..) => todo!("{value:?}"),
        ir::Value::Match(..) => todo!("{value:?}"),
        ir::Value::Fn(..) => todo!("{value:?}"),
        ir::Value::Do(..) => todo!("{value:?}"),
        ir::Value::Handled(..) => todo!("{value:?}"),
        ir::Value::Module(..) => todo!("{value:?}"),
        ir::Value::Reference(ident) => {
            let binding = context.scope.lookup(&ident.id);
            match binding {
                Some(Binding::Constant(value)) => {
                    context.write_instruction(Instruction::Const(value.clone()));
                }
                Some(&Binding::Variable(offset)) => {
                    context.write_instruction(Instruction::LoadLocal(offset));
                }
                Some(Binding::Label(label)) => {
                    context.write_procedure_reference(label.to_owned());
                }
                None => panic!("Unresolved reference should not exist at this point"),
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
