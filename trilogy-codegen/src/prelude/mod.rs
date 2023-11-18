mod chunk_writer_ext;
mod label_maker;
mod stack_tracker;
mod stateful_chunk_writer_ext;
mod type_pattern;

pub(crate) use crate::context::{Binding, Context};
pub(crate) use crate::function::write_function;
pub(crate) use crate::module::{write_module_definitions, write_module_prelude};
pub(crate) use crate::operator::*;
pub(crate) use crate::preamble::*;
pub(crate) use crate::procedure::write_procedure;
pub(crate) use crate::query::*;
pub(crate) use crate::rule::write_rule;
pub(crate) use chunk_writer_ext::*;
pub(crate) use label_maker::*;
pub(crate) use stack_tracker::*;
pub(crate) use stateful_chunk_writer_ext::*;
pub(crate) use type_pattern::*;

use trilogy_ir::ir;
use trilogy_vm::Offset;

pub const HANDLER: Offset = 0;
pub const MODULE: Offset = 1;
pub const BINDSET: Offset = 2;
pub const TEMPORARY: Offset = 3;

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
