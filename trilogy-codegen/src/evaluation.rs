use crate::context::Binding;
use crate::{is_operator, write_operator, Context};
use trilogy_ir::ir;
use trilogy_vm::{Instruction, Value};

#[allow(clippy::only_used_in_recursion)]
pub(crate) fn write_evaluation(context: &mut Context, expr: &ir::Expression) {
    match &expr.value {
        ir::Value::Builtin(..) => todo!(),
        ir::Value::Pack(pack) => {
            for element in &pack.values {
                if element.is_spread {
                    todo!()
                } else {
                    write_evaluation(context, &element.expression);
                }
            }
        }
        ir::Value::Sequence(seq) => {
            for expr in seq {
                write_evaluation(context, expr);
            }
        }
        ir::Value::Assignment(..) => todo!(),
        ir::Value::Mapping(..) => todo!(),
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
        ir::Value::Application(application) => {
            match &application.function.value {
                ir::Value::Builtin(builtin) if is_operator(*builtin) => {
                    return write_unary_operation(context, &application.argument, *builtin)
                }
                ir::Value::Application(lhs_app) => match &lhs_app.function.value {
                    ir::Value::Builtin(builtin) if is_operator(*builtin) => {
                        return write_binary_operation(
                            context,
                            &lhs_app.argument,
                            &application.argument,
                            *builtin,
                        )
                    }
                    _ => {}
                },
                _ => {}
            }
            write_evaluation(context, &application.function);
            let start = context.stack_height;
            write_evaluation(context, &application.argument);
            let end = context.stack_height;
            // TODO: support multiple arguments more efficiently?
            context.write_instruction(Instruction::Call(end - start));
        }
        ir::Value::Let(..) => todo!(),
        ir::Value::IfElse(..) => todo!(),
        ir::Value::Match(..) => todo!(),
        ir::Value::Fn(..) => todo!(),
        ir::Value::Do(..) => todo!(),
        ir::Value::Handled(..) => todo!(),
        ir::Value::Module(..) => todo!(),
        ir::Value::Reference(ident) => {
            let binding = context.scope.lookup(&ident.id);
            match binding {
                Some(Binding::Constant(value)) => {
                    context.write_instruction(Instruction::Const(value.clone()));
                }
                Some(&Binding::Variable(offset)) => {
                    context.write_instruction(Instruction::LoadRegister(offset));
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

fn write_unary_operation(context: &mut Context, value: &ir::Expression, builtin: ir::Builtin) {
    write_evaluation(context, value);
    write_operator(context, builtin);
}

fn write_binary_operation(
    context: &mut Context,
    lhs: &ir::Expression,
    rhs: &ir::Expression,
    builtin: ir::Builtin,
) {
    write_evaluation(context, lhs);
    write_evaluation(context, rhs);
    write_operator(context, builtin);
}
