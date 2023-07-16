use crate::context::{Binding, Context};
use trilogy_ir::ir::{self, Expression};
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
        ir::Value::Conjunction(..) => todo!(),
        ir::Value::Disjunction(..) => todo!(),
        ir::Value::Wildcard => {} // Wildcard always matches, so is noop
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
                context
                    .scope
                    .declare_variable(ident.id.clone(), context.stack_height);
                context.write_instruction(Instruction::LoadRegister(register));
            }
        },
        ir::Value::Dynamic(dynamic) => {
            panic!("Dynamic is not actually supposed to happen, but we got {dynamic:?}");
        }
        _ => panic!("{:?} is not a pattern", expr.value),
    }
}
