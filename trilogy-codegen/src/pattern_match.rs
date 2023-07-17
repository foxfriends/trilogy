use crate::context::{Binding, Context};
use trilogy_ir::ir::{self, Builtin, Expression};
use trilogy_vm::{Instruction, Value};

/// Pattern matches the contents of a particular register with an expression.
///
/// On success, the stack now includes the bindings of the expression in separate registers.
/// On failure, the provided label is jumped to.
/// In either case, the original value is left unchanged.
pub(crate) fn write_pattern_match(
    context: &mut Context,
    register: usize,
    expr: &Expression,
    on_fail: &str,
) {
    match &expr.value {
        ir::Value::Mapping(..) => todo!(),
        ir::Value::Number(value) => {
            context
                .write_instruction(Instruction::LoadRegister(register))
                .write_instruction(Instruction::Const(value.value().clone().into()))
                .write_instruction(Instruction::ValEq)
                .cond_jump(on_fail);
        }
        ir::Value::Character(value) => {
            context
                .write_instruction(Instruction::LoadRegister(register))
                .write_instruction(Instruction::Const((*value).into()))
                .write_instruction(Instruction::ValEq)
                .cond_jump(on_fail);
        }
        ir::Value::String(value) => {
            context
                .write_instruction(Instruction::LoadRegister(register))
                .write_instruction(Instruction::Const(value.into()))
                .write_instruction(Instruction::ValEq)
                .cond_jump(on_fail);
        }
        ir::Value::Bits(value) => {
            context
                .write_instruction(Instruction::LoadRegister(register))
                .write_instruction(Instruction::Const(value.value().clone().into()))
                .write_instruction(Instruction::ValEq)
                .cond_jump(on_fail);
        }
        ir::Value::Boolean(value) => {
            context
                .write_instruction(Instruction::LoadRegister(register))
                .write_instruction(Instruction::Const((*value).into()))
                .write_instruction(Instruction::ValEq)
                .cond_jump(on_fail);
        }
        ir::Value::Unit => {
            context
                .write_instruction(Instruction::LoadRegister(register))
                .write_instruction(Instruction::Const(Value::Unit))
                .write_instruction(Instruction::ValEq)
                .cond_jump(on_fail);
        }
        ir::Value::Conjunction(conj) => {
            write_pattern_match(context, register, &conj.0, on_fail);
            // TODO: cleanup if second one fails
            write_pattern_match(context, register, &conj.1, on_fail);
        }
        ir::Value::Disjunction(disj) => {
            let next_pattern = format!(
                "disj::middle::{:x}",
                (&**disj) as *const (ir::Expression, ir::Expression) as usize
            );
            let success = format!(
                "disj::end::{:x}",
                (&**disj) as *const (ir::Expression, ir::Expression) as usize
            );
            write_pattern_match(context, register, &disj.0, &next_pattern);
            context.jump(&success);
            context
                .write_label(next_pattern)
                .expect("disjunction pointer is unique");
            write_pattern_match(context, register, &disj.1, on_fail);
            context
                .write_label(success)
                .expect("disjunction pointer is unique");
        }
        ir::Value::Wildcard => {} // Wildcard always matches but does not bind, so is noop
        ir::Value::Atom(value) => {
            let atom = context.atom(value);
            context
                .write_instruction(Instruction::LoadRegister(register))
                .write_instruction(Instruction::Const(atom.into()))
                .write_instruction(Instruction::ValEq)
                .cond_jump(on_fail);
        }
        ir::Value::Reference(ident) => match context.scope.lookup(&ident.id) {
            Some(Binding::Constant(value)) => {
                let value = value.clone();
                context
                    .write_instruction(Instruction::LoadRegister(register))
                    .write_instruction(Instruction::Const(value))
                    .write_instruction(Instruction::ValEq)
                    .cond_jump(on_fail);
            }
            Some(&Binding::Variable(offset)) => {
                context
                    .write_instruction(Instruction::LoadRegister(register))
                    .write_instruction(Instruction::LoadRegister(offset))
                    .write_instruction(Instruction::ValEq)
                    .cond_jump(on_fail);
            }
            Some(Binding::Label(label)) => {
                let label = label.clone();
                context
                    .write_instruction(Instruction::LoadRegister(register))
                    .write_procedure_reference(label)
                    .write_instruction(Instruction::ValEq)
                    .cond_jump(on_fail);
            }
            None => {
                context.scope.declare_variable(ident.id.clone(), register);
            }
        },
        ir::Value::Application(application) => match &application.function.value {
            ir::Value::Builtin(Builtin::Negate) => {
                context
                    .write_instruction(Instruction::LoadRegister(register))
                    .write_instruction(Instruction::Negate);
                write_pattern_match(
                    context,
                    context.stack_height,
                    &application.argument,
                    on_fail,
                );
            }
            ir::Value::Application(lhs_app) => match &lhs_app.function.value {
                ir::Value::Builtin(Builtin::Glue) => todo!(),
                ir::Value::Builtin(Builtin::Construct) => {
                    context
                        .write_instruction(Instruction::LoadRegister(register))
                        .write_instruction(Instruction::Destruct);
                    let atom = context.stack_height;
                    let value = context.stack_height - 1;
                    write_pattern_match(context, atom, &application.argument, on_fail);
                    // TODO: cleanup if second one fails
                    write_pattern_match(context, value, &lhs_app.argument, on_fail);
                }
                ir::Value::Builtin(Builtin::Cons) => {
                    context
                        .write_instruction(Instruction::LoadRegister(register))
                        .write_instruction(Instruction::Uncons);
                    let rhs = context.stack_height;
                    let lhs = context.stack_height - 1;
                    write_pattern_match(context, rhs, &application.argument, on_fail);
                    // TODO: cleanup if second one fails
                    write_pattern_match(context, lhs, &lhs_app.argument, on_fail);
                }
                ir::Value::Builtin(Builtin::Array) => todo!(),
                ir::Value::Builtin(Builtin::Record) => todo!(),
                ir::Value::Builtin(Builtin::Set) => todo!(),
                _ => panic!("not a pattern"),
            },
            _ => panic!("not a pattern"),
        },
        ir::Value::Dynamic(dynamic) => {
            panic!("Dynamic is not actually supposed to happen, but we got {dynamic:?}");
        }
        _ => panic!("{:?} is not a pattern", expr.value),
    }
}
