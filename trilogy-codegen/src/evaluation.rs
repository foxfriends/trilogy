use crate::labeler::Labeler;
use trilogy_ir::ir::{self, Builtin};
use trilogy_vm::{Instruction, ProgramBuilder, Value};

#[allow(clippy::only_used_in_recursion)]
pub(crate) fn write_evaluation(
    labeler: &mut Labeler,
    builder: &mut ProgramBuilder,
    expr: &ir::Expression,
) {
    match &expr.value {
        ir::Value::Builtin(Builtin::Negate) => todo!(),
        ir::Value::Builtin(Builtin::Not) => todo!(),
        ir::Value::Builtin(Builtin::Access) => todo!(),
        ir::Value::Builtin(Builtin::And) => todo!(),
        ir::Value::Builtin(Builtin::Or) => todo!(),
        ir::Value::Builtin(Builtin::Add) => todo!(),
        ir::Value::Builtin(Builtin::Subtract) => todo!(),
        ir::Value::Builtin(Builtin::Multiply) => todo!(),
        ir::Value::Builtin(Builtin::Divide) => todo!(),
        ir::Value::Builtin(Builtin::Remainder) => todo!(),
        ir::Value::Builtin(Builtin::Power) => todo!(),
        ir::Value::Builtin(Builtin::IntDivide) => todo!(),
        ir::Value::Builtin(Builtin::StructuralEquality) => todo!(),
        ir::Value::Builtin(Builtin::StructuralInequality) => todo!(),
        ir::Value::Builtin(Builtin::ReferenceEquality) => todo!(),
        ir::Value::Builtin(Builtin::ReferenceInequality) => todo!(),
        ir::Value::Builtin(Builtin::Lt) => todo!(),
        ir::Value::Builtin(Builtin::Gt) => todo!(),
        ir::Value::Builtin(Builtin::Leq) => todo!(),
        ir::Value::Builtin(Builtin::Geq) => todo!(),
        ir::Value::Builtin(Builtin::BitwiseAnd) => todo!(),
        ir::Value::Builtin(Builtin::BitwiseOr) => todo!(),
        ir::Value::Builtin(Builtin::BitwiseXor) => todo!(),
        ir::Value::Builtin(Builtin::LeftShift) => todo!(),
        ir::Value::Builtin(Builtin::RightShift) => todo!(),
        ir::Value::Builtin(Builtin::Sequence) => todo!(),
        ir::Value::Builtin(Builtin::Cons) => todo!(),
        ir::Value::Builtin(Builtin::Glue) => todo!(),
        ir::Value::Builtin(Builtin::Invert) => todo!(),
        ir::Value::Builtin(Builtin::ModuleAccess) => todo!(),
        ir::Value::Builtin(Builtin::Compose) => todo!(),
        ir::Value::Builtin(Builtin::RCompose) => todo!(),
        ir::Value::Builtin(Builtin::Pipe) => todo!(),
        ir::Value::Builtin(Builtin::RPipe) => todo!(),
        ir::Value::Builtin(Builtin::Construct) => todo!(),
        ir::Value::Builtin(Builtin::Array) => todo!(),
        ir::Value::Builtin(Builtin::Set) => todo!(),
        ir::Value::Builtin(Builtin::Record) => todo!(),
        ir::Value::Builtin(Builtin::Is) => todo!(),
        ir::Value::Builtin(Builtin::For) => todo!(),
        ir::Value::Builtin(Builtin::Yield) => todo!(),
        ir::Value::Builtin(Builtin::Resume) => todo!(),
        ir::Value::Builtin(Builtin::Cancel) => todo!(),
        ir::Value::Builtin(Builtin::Return) => todo!(),
        ir::Value::Builtin(Builtin::Break) => todo!(),
        ir::Value::Builtin(Builtin::Continue) => todo!(),
        ir::Value::Builtin(Builtin::Exit) => todo!(),
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
        ir::Value::Conjunction(..) => todo!(),
        ir::Value::Disjunction(..) => todo!(),
        ir::Value::Wildcard => unreachable!("Wildcard cannot appear in this location"),
        ir::Value::Atom(value) => {
            let atom = builder.atom(value);
            builder.write_instruction(Instruction::Const(atom.into()));
        }
        ir::Value::Query(..) => todo!(),
        ir::Value::Iterator(..) => todo!(),
        ir::Value::While(..) => todo!(),
        ir::Value::Application(application) => {
            write_evaluation(labeler, builder, &application.function);
            write_evaluation(labeler, builder, &application.argument);
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
