mod context;
mod evaluation;
mod function;
mod helpers;
mod module;
mod operator;
mod pattern_match;
mod preamble;
mod procedure;
mod program;
mod query;
mod rule;
mod static_expression;

pub use program::write_program;

mod prelude {
    pub(crate) use crate::context::{Binding, Context};
    pub(crate) use crate::evaluation::{write_evaluation, write_expression};
    pub(crate) use crate::function::write_function;
    pub(crate) use crate::helpers::*;
    pub(crate) use crate::module::write_module;
    pub(crate) use crate::operator::*;
    pub(crate) use crate::pattern_match::write_pattern_match;
    pub(crate) use crate::preamble::write_preamble;
    pub(crate) use crate::procedure::write_procedure;
    pub(crate) use crate::query::*;
    pub(crate) use crate::rule::write_rule;
    pub(crate) use crate::static_expression::{write_static_expression, write_static_value};
}
