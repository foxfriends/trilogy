use crate::{
    context::{Binding, Context},
    evaluation::write_evaluation,
};
use trilogy_ir::ir::{self, Builtin, Expression};
use trilogy_vm::{Instruction, Value};

/// Pattern matches the contents of a particular register with an expression.
///
/// On success, the stack now includes the bindings of the expression in separate registers.
/// On failure, the provided label is jumped to.
/// In either case, the original value is left unchanged.
pub(crate) fn write_pattern_match(context: &mut Context, expr: &Expression, on_fail: &mut String) {
    match &expr.value {
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
            write_pattern_match(context, &conj.0, on_fail);
            write_pattern_match(context, &conj.1, on_fail);
        }
        ir::Value::Disjunction(disj) => {
            let next = context.labeler.unique();
            let recover = std::mem::replace(on_fail, context.labeler.unique());
            context.write_instruction(Instruction::Copy);
            write_pattern_match(context, &disj.0, on_fail);
            context
                .write_instruction(Instruction::Pop)
                .jump(&next)
                .write_label(recover)
                .unwrap();
            write_pattern_match(context, &disj.1, on_fail);
            context.write_label(next).unwrap();
        }
        ir::Value::Wildcard => {} // Wildcard always matches but does not bind, so is noop
        ir::Value::Atom(value) => {
            let atom = context.atom(value);
            context
                .write_instruction(Instruction::Const(atom.into()))
                .write_instruction(Instruction::ValEq)
                .cond_jump(on_fail);
        }
        ir::Value::Reference(ident) => match context.scope.lookup(&ident.id).unwrap() {
            Binding::Constant(value) => {
                let value = value.clone();
                context
                    .write_instruction(Instruction::Const(value))
                    .write_instruction(Instruction::ValEq)
                    .cond_jump(on_fail);
            }
            Binding::Variable(offset) => {
                context.write_instruction(Instruction::SetLocal(*offset));
            }
            Binding::Label(label) => {
                let label = label.clone();
                context
                    .write_procedure_reference(label)
                    .write_instruction(Instruction::ValEq)
                    .cond_jump(on_fail);
            }
        },
        ir::Value::Application(application) => match &application.function.value {
            ir::Value::Builtin(Builtin::Negate) => {
                context.write_instruction(Instruction::Negate);
                write_pattern_match(context, &application.argument, on_fail);
            }
            ir::Value::Builtin(Builtin::Pin) => {
                write_evaluation(context, &application.argument);
                context
                    .write_instruction(Instruction::ValEq)
                    .cond_jump(on_fail);
            }
            ir::Value::Application(lhs_app) => match &lhs_app.function.value {
                ir::Value::Builtin(Builtin::Glue) => todo!(),
                ir::Value::Builtin(Builtin::Construct) => {
                    context.write_instruction(Instruction::Destruct);
                    let next = context.labeler.unique();
                    let cleanup = std::mem::replace(on_fail, context.labeler.unique());
                    write_pattern_match(context, &application.argument, on_fail);
                    write_pattern_match(context, &lhs_app.argument, on_fail);
                    context
                        .jump(&next)
                        .write_label(cleanup)
                        .unwrap()
                        .write_instruction(Instruction::Pop)
                        .write_instruction(Instruction::Pop)
                        .jump(on_fail)
                        .write_label(next)
                        .unwrap();
                }
                ir::Value::Builtin(Builtin::Cons) => {
                    context.write_instruction(Instruction::Uncons);
                    let next = context.labeler.unique();
                    let cleanup = std::mem::replace(on_fail, context.labeler.unique());
                    write_pattern_match(context, &application.argument, on_fail);
                    write_pattern_match(context, &lhs_app.argument, on_fail);
                    context
                        .jump(&next)
                        .write_label(cleanup)
                        .unwrap()
                        .write_instruction(Instruction::Pop)
                        .write_instruction(Instruction::Pop)
                        .jump(on_fail)
                        .write_label(next)
                        .unwrap();
                }
                ir::Value::Builtin(Builtin::Array) => todo!(),
                ir::Value::Builtin(Builtin::Record) => todo!(),
                ir::Value::Builtin(Builtin::Set) => todo!(),
                what => panic!("not a pattern ({what:?})"),
            },
            what => panic!("not a pattern ({what:?})"),
        },
        ir::Value::Dynamic(dynamic) => {
            panic!("Dynamic is not actually supposed to happen, but we got {dynamic:?}");
        }
        _ => panic!("{:?} is not a pattern", expr.value),
    }
}
