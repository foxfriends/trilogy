mod context;
mod entrypoint;
mod evaluation;
mod function;
mod helpers;
mod module;
mod operator;
mod pattern_match;
mod preamble;
mod procedure;
mod query;
mod rule;

pub use entrypoint::{write_module, write_program};
pub use preamble::RETURN;

mod prelude {
    pub(crate) use crate::context::{Binding, Context};
    pub(crate) use crate::evaluation::{write_evaluation, write_expression};
    pub(crate) use crate::function::write_function;
    pub(crate) use crate::helpers::*;
    pub(crate) use crate::module::{write_module_definitions, write_module_prelude};
    pub(crate) use crate::operator::*;
    pub(crate) use crate::pattern_match::write_pattern_match;
    pub(crate) use crate::preamble::write_preamble;
    pub(crate) use crate::procedure::write_procedure;
    pub(crate) use crate::query::*;
    pub(crate) use crate::rule::write_rule;
}
