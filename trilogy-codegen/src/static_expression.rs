use crate::context::Context;
use trilogy_ir::ir;

pub(crate) fn write_static_expression(context: &mut Context, expression: &ir::Expression) {
    write_static_value(context, &expression.value)
}

pub(crate) fn write_static_value(_context: &mut Context, _value: &ir::Value) {
    todo!()
}
