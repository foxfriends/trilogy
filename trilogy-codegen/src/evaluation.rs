use crate::{is_operator, write_operator, Labeler};
use trilogy_ir::ir;
use trilogy_vm::{Instruction, ProgramBuilder, Value};

#[allow(clippy::only_used_in_recursion)]
pub(crate) fn write_evaluation(
    labeler: &mut Labeler,
    builder: &mut ProgramBuilder,
    expr: &ir::Expression,
) {
    match &expr.value {
        ir::Value::Builtin(..) => todo!(),
        ir::Value::Pack(..) => todo!(),
        ir::Value::Sequence(seq) => {
            for expr in seq {
                write_evaluation(labeler, builder, expr);
            }
        }
        ir::Value::Assignment(..) => todo!(),
        ir::Value::Mapping(..) => todo!(),
        ir::Value::Number(value) => {
            builder.write_instruction(Instruction::Const(value.value().clone().into()));
        }
        ir::Value::Character(value) => {
            builder.write_instruction(Instruction::Const((*value).into()));
        }
        ir::Value::String(value) => {
            builder.write_instruction(Instruction::Const(value.into()));
        }
        ir::Value::Bits(value) => {
            builder.write_instruction(Instruction::Const(value.value().clone().into()));
        }
        ir::Value::Boolean(value) => {
            builder.write_instruction(Instruction::Const((*value).into()));
        }
        ir::Value::Unit => {
            builder.write_instruction(Instruction::Const(Value::Unit));
        }
        ir::Value::Conjunction(..) => unreachable!("Conjunction cannot appear in an evaluation"),
        ir::Value::Disjunction(..) => unreachable!("Disjunction cannot appear in an evaluation"),
        ir::Value::Wildcard => unreachable!("Wildcard cannot appear in an evaluation"),
        ir::Value::Atom(value) => {
            let atom = builder.atom(value);
            builder.write_instruction(Instruction::Const(atom.into()));
        }
        ir::Value::Query(..) => todo!(),
        ir::Value::Iterator(..) => todo!(),
        ir::Value::While(..) => todo!(),
        ir::Value::Application(application) => {
            match &application.function.value {
                ir::Value::Builtin(builtin) if is_operator(*builtin) => {
                    return write_unary_operation(labeler, builder, &application.argument, *builtin)
                }
                ir::Value::Application(lhs_app) => match &lhs_app.function.value {
                    ir::Value::Builtin(builtin) if is_operator(*builtin) => {
                        return write_binary_operation(
                            labeler,
                            builder,
                            &lhs_app.argument,
                            &application.argument,
                            *builtin,
                        )
                    }
                    _ => {}
                },
                _ => {}
            }
            write_evaluation(labeler, builder, &application.function);
            write_evaluation(labeler, builder, &application.argument);
            // TODO: support multiple arguments more efficiently?
            builder.write_instruction(Instruction::Call(1));
        }
        ir::Value::Let(..) => todo!(),
        ir::Value::IfElse(..) => todo!(),
        ir::Value::Match(..) => todo!(),
        ir::Value::Fn(..) => todo!(),
        ir::Value::Do(..) => todo!(),
        ir::Value::Handled(..) => todo!(),
        ir::Value::Module(..) => todo!(),
        ir::Value::Reference(..) => todo!(),
        ir::Value::Dynamic(dynamic) => {
            panic!("Dynamic is not actually supposed to happen, but we got {dynamic:?}");
        }
        ir::Value::Assert(..) => todo!(),
        ir::Value::End => {
            builder.write_instruction(Instruction::Fizzle);
        }
    }
}

fn write_unary_operation(
    labeler: &mut Labeler,
    builder: &mut ProgramBuilder,
    value: &ir::Expression,
    builtin: ir::Builtin,
) {
    write_evaluation(labeler, builder, value);
    write_operator(labeler, builder, builtin);
}

fn write_binary_operation(
    labeler: &mut Labeler,
    builder: &mut ProgramBuilder,
    lhs: &ir::Expression,
    rhs: &ir::Expression,
    builtin: ir::Builtin,
) {
    write_evaluation(labeler, builder, lhs);
    write_evaluation(labeler, builder, rhs);
    write_operator(labeler, builder, builtin);
}
