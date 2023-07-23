use trilogy_ir::ir;

pub(crate) fn unapply_2(
    application: &ir::Application,
) -> (Option<&ir::Value>, &ir::Value, &ir::Value) {
    match &application.function.value {
        ir::Value::Application(lhs) => (
            Some(&lhs.function.value),
            &lhs.argument.value,
            &application.argument.value,
        ),
        _ => (
            None,
            &application.function.value,
            &application.argument.value,
        ),
    }
}
