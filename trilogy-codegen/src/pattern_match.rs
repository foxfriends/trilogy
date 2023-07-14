use crate::context::Context;
use trilogy_ir::ir::Expression;

/// Pattern matches the contents of a particular register with an expression.
///
/// On success, the stack now includes the bindings of the expression in separate registers.
/// On failure, the provided label is jumped to.
/// In either case, the original value is left unchanged.
pub(crate) fn write_pattern_match(
    context: &mut Context,
    register: usize,
    expression: &Expression,
    on_fail: &str,
) {
    todo!("this will be ... fun")
}
